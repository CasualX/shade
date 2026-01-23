use std::ffi::CString;
use std::num::NonZeroU32;
use std::collections::HashSet;

use glutin::prelude::*;
use shade::cvmath::*;

const PICK_RADIUS_PX: f32 = 12.0;
const NO_DRAG: usize = usize::MAX;

struct PolygonDemo {
	color_shader: shade::ShaderProgram,
	points: Vec<Point2f>,
	cursor_px: Vec2f,
	view_offset: Vec2f,
	panning: bool,
	pan_start_px: Vec2f,
	dragging: usize,
	drag_offset_px: Vec2f,
}

impl PolygonDemo {
	fn new(g: &mut shade::Graphics) -> PolygonDemo {
		let color_shader = g.shader_compile(shade::gl::shaders::COLOR_VS, shade::gl::shaders::COLOR_FS);
		PolygonDemo {
			color_shader,
			points: Vec::new(),
			cursor_px: Vec2f::ZERO,
			view_offset: Vec2f::ZERO,
			panning: false,
			pan_start_px: Vec2f::ZERO,
			dragging: NO_DRAG,
			drag_offset_px: Vec2f::ZERO,
		}
	}

	fn cursor_world(&self) -> Point2f {
		Point2(self.cursor_px.x + self.view_offset.x, self.cursor_px.y + self.view_offset.y)
	}

	fn pick_point_index(&self) -> Option<usize> {
		let mut best: Option<(usize, f32)> = None;
		let r2 = PICK_RADIUS_PX * PICK_RADIUS_PX;
		let cursor = self.cursor_world();
		for (i, p) in self.points.iter().enumerate() {
			let dist2 = p.distance_sqr(cursor);
			if dist2 <= r2 {
				match best {
					None => best = Some((i, dist2)),
					Some((_, best2)) if dist2 < best2 => best = Some((i, dist2)),
					_ => {}
				}
			}
		}
		best.map(|(i, _)| i)
	}

	fn start_drag_index(&mut self, idx: usize) {
		self.dragging = idx;
		self.drag_offset_px = Vec2f::ZERO;
	}

	fn begin_drag(&mut self) -> bool {
		if self.dragging != NO_DRAG {
			return true;
		}
		if let Some(i) = self.pick_point_index() {
			self.start_drag_index(i);
			return true;
		}
		false
	}

	fn pick_edge_insertion(&self) -> Option<(usize, Point2f)> {
		let n = self.points.len();
		if n < 2 {
			return None;
		}

		let close = n >= 3;
		let mut best: Option<(usize, Point2f, f32)> = None;
		let cursor = self.cursor_world();

		let seg_count = if close { n } else { n - 1 };
		for i in 0..seg_count {
			let seg = Line2(self.points[i], self.points[(i + 1) % n]);
			let dist = seg.distance(cursor);
			let q = seg.project(cursor);

			let insert_index = if close && i == n - 1 { n } else { i + 1 };
			let dist2 = dist * dist;
			match best {
				None => best = Some((insert_index, q, dist2)),
				Some((_, _, best2)) if dist2 < best2 => best = Some((insert_index, q, dist2)),
				_ => {}
			}
		}

		best.map(|(idx, q, _)| (idx, q))
	}

	fn split_edge_at_cursor(&mut self) -> usize {
		let (insert_index, q) = self.pick_edge_insertion().expect("split_edge_at_cursor requires at least 2 points");

		self.points.insert(insert_index, q);
		insert_index
	}

	fn end_drag(&mut self) {
		self.dragging = NO_DRAG;
	}

	fn drag_to_cursor(&mut self) {
		let i = self.dragging;
		if i == NO_DRAG {
			return;
		}
		if i >= self.points.len() {
			self.end_drag();
			return;
		}

		self.points[i] = self.cursor_world();
	}

	fn delete_point_at_cursor(&mut self) -> bool {
		let Some(idx) = self.pick_point_index() else { return false; };

		// If we're dragging, keep the drag state consistent.
		if self.dragging != NO_DRAG {
			if self.dragging == idx {
				self.end_drag();
			} else if idx < self.dragging {
				self.dragging -= 1;
			}
		}

		self.points.remove(idx);
		true
	}

	fn add_point_from_cursor(&mut self) -> usize {
		self.points.push(self.cursor_world());
		self.points.len() - 1
	}

	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		shade::clear!(g, color: Vec4(0.08, 0.09, 0.11, 1.0));

		let mut cv = shade::d2::ColorBuffer::new();
		cv.shader = self.color_shader;
		cv.blend_mode = shade::BlendMode::Alpha;
		let w = (viewport.maxs.x - viewport.mins.x) as f32;
		let h = (viewport.maxs.y - viewport.mins.y) as f32;
		let world = Bounds2::c(
			self.view_offset.x,
			self.view_offset.y,
			self.view_offset.x + w,
			self.view_offset.y + h,
		);
		cv.uniform.transform = Transform2::ortho(world);

		let fill_paint = shade::d2::Paint {
			template: shade::d2::ColorTemplate {
				color1: Vec4(40, 180, 255, 120),
				color2: Vec4(40, 180, 255, 120),
			},
		};
		let outline_pen = shade::d2::Pen {
			template: shade::d2::ColorTemplate {
				color1: Vec4(240, 240, 240, 220),
				color2: Vec4(240, 240, 240, 220),
			},
		};
		let point_paint = shade::d2::Paint {
			template: shade::d2::ColorTemplate {
				color1: Vec4(255, 210, 0, 255),
				color2: Vec4(255, 210, 0, 255),
			},
		};
		let tri_pen = shade::d2::Pen {
			template: shade::d2::ColorTemplate {
				color1: Vec4(255, 255, 255, 80),
				color2: Vec4(255, 255, 255, 80),
			},
		};

		if self.points.len() >= 3 {
			let tris = shade::d2::polygon::triangulate(&self.points);
			cv.fill_polygon(&fill_paint, &self.points, &tris);

			// Triangulation wireframe (including internal edges)
			let mut edges: HashSet<(u32, u32)> = HashSet::new();
			for &shade::Index3 { p1, p2, p3 } in &tris {
				let (ab0, ab1) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
				let (bc0, bc1) = if p2 < p3 { (p2, p3) } else { (p3, p2) };
				let (ca0, ca1) = if p3 < p1 { (p3, p1) } else { (p1, p3) };
				edges.insert((ab0, ab1));
				edges.insert((bc0, bc1));
				edges.insert((ca0, ca1));
			}
			let lines: Vec<(u32, u32)> = edges.into_iter().collect();
			cv.draw_lines(&tri_pen, &self.points, &lines);
		}

		if self.points.len() >= 2 {
			let close = self.points.len() >= 3;
			cv.draw_poly_line(&outline_pen, &self.points, close);
		}

		for p in &self.points {
			let r = 4.0;
			let rc = Bounds2::c(p.x - r, p.y - r, p.x + r, p.y + r);
			cv.fill_ellipse(&point_paint, &rc, 12);
		}

		cv.draw(g);
		g.end();
	}
}

/// OpenGL Window wrapper.
struct GlWindow {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
}

impl GlWindow {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> GlWindow {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let template_builder = ConfigTemplateBuilder::new().with_alpha_size(8).with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default().with_inner_size(size);

		let config_picker = |configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>| {
			configs
				.filter(|c| c.srgb_capable())
				.max_by_key(|c| c.num_samples())
				.expect("No GL configs found")
		};
		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template_builder, config_picker)
			.expect("Failed DisplayBuilder.build");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window
			.window_handle()
			.expect("Failed Window.window_handle")
			.as_raw();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe { gl_display.create_context(&gl_config, &context_attributes) }
			.expect("Failed Display.create_context");

		let surface_attributes_builder = SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(Some(true));
		let surface_attributes = surface_attributes_builder.build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");

		let context = not_current
			.make_current(&surface)
			.expect("Failed NotCurrentContext.make_current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		GlWindow {
			size,
			window,
			surface,
			context,
		}
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
	}
}

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: PolygonDemo,
	dirty: bool,
	shift: bool,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = PolygonDemo::new(opengl.as_graphics());
		Box::new(App {
			window,
			opengl,
			demo,
			dirty: true,
			shift: false,
		})
	}

	fn draw(&mut self) {
		let viewport = Bounds2::c(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
		self.dirty = false;
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
	let size = winit::dpi::PhysicalSize::new(900, 700);

	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
			use winit::event::{ElementState, Event, MouseButton, WindowEvent};

		match event {
			Event::Resumed => {
				if app.is_none() {
					app = Some(App::new(event_loop, size));
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::ModifiersChanged(mods) => {
					if let Some(app) = app.as_deref_mut() {
						// winit 0.30: `mods` is a Modifiers type with a `.state()` accessor.
						app.shift = mods.state().shift_key();
					}
				}
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.window.resize(new_size);
						app.dirty = true;
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.demo.cursor_px.x = position.x as f32;
						app.demo.cursor_px.y = position.y as f32;

						if app.demo.panning {
							let delta = app.demo.cursor_px - app.demo.pan_start_px;
							app.demo.view_offset -= delta;
							app.demo.pan_start_px = app.demo.cursor_px;
							app.dirty = true;
							app.window.window.request_redraw();
							return;
						}

						if app.demo.dragging != NO_DRAG {
							app.demo.drag_to_cursor();
							app.dirty = true;
							app.window.window.request_redraw();
						}
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						match (button, state) {
							(MouseButton::Left, ElementState::Pressed) => {
								if app.shift {
									app.demo.panning = true;
									app.demo.pan_start_px = app.demo.cursor_px;
									app.demo.end_drag();
									app.dirty = true;
									app.window.window.request_redraw();
									return;
								}

								if !app.demo.begin_drag() {
									if app.demo.points.len() < 2 {
										let idx = app.demo.add_point_from_cursor();
										app.demo.start_drag_index(idx);
									}
									else {
										let idx = app.demo.split_edge_at_cursor();
										app.demo.start_drag_index(idx);
									}
								}
								app.demo.drag_to_cursor();
								app.dirty = true;
								app.window.window.request_redraw();
							}
							(MouseButton::Left, ElementState::Released) => {
								if app.demo.panning {
									app.demo.panning = false;
									return;
								}
								app.demo.end_drag();
							}
							(MouseButton::Right, ElementState::Pressed) => {
								if app.demo.delete_point_at_cursor() {
									app.dirty = true;
									app.window.window.request_redraw();
								}
							}
							_ => {}
						}
					}
				}
				WindowEvent::CloseRequested => event_loop.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						app.draw();
						app.window.surface.swap_buffers(&app.window.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					if app.dirty {
						app.window.window.request_redraw();
					}
				}
			}
			_ => {}
		}
	});
}
