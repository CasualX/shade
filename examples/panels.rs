use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;
use shade::d2;

const INFLATE_PX: i32 = 5;

//----------------------------------------------------------------
// OpenGL window wrapper

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

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_title("shade pixel art")
			.with_inner_size(size);

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

		let surface_attributes_builder = SurfaceAttributesBuilder::<WindowSurface>::new()
			.with_srgb(Some(true));
		let surface_attributes = surface_attributes_builder.build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");

		let context = not_current.make_current(&surface)
			.expect("Failed NotCurrentContext.make_current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		GlWindow { size, window, surface, context }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
	}

}

//----------------------------------------------------------------
// Demo state

#[derive(Copy, Clone, Eq, PartialEq)]
enum DragEdge {
	Move,
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,
	Left,
	Right,
	Top,
	Bottom,
}

#[derive(Copy, Clone, PartialEq)]
struct PanelInstance {
	texture: shade::Texture2D,
	shader: shade::ShaderProgram,
	uv_x: &'static [f32],
	uv_y: &'static [f32],
	template_x: &'static [d2::layout::Unit],
	template_y: &'static [d2::layout::Unit],
	bounds: Bounds2i,
	min_size: Vec2i,
}

impl PanelInstance {
	fn hit_test(&self, pos: Vec2i) -> Option<DragEdge> {
		if Bounds2::point(self.bounds.top_left(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::TopLeft);
		}
		if Bounds2::point(self.bounds.top_right(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::TopRight);
		}
		if Bounds2::point(self.bounds.bottom_left(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::BottomLeft);
		}
		if Bounds2::point(self.bounds.bottom_right(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::BottomRight);
		}
		fn vertical(v: i32, b: Bounds2i) -> Bounds2i {
			let x_min = v - INFLATE_PX;
			let x_max = v + INFLATE_PX;
			let y_min = b.top();
			let y_max = b.bottom();
			Bounds2(Point2(x_min, y_min), Point2(x_max, y_max))
		}
		fn horizontal(v: i32, b: Bounds2i) -> Bounds2i {
			let x_min = b.left();
			let x_max = b.right();
			let y_min = v - INFLATE_PX;
			let y_max = v + INFLATE_PX;
			Bounds2(Point2(x_min, y_min), Point2(x_max, y_max))
		}
		if vertical(self.bounds.left(), self.bounds).contains(pos) {
			return Some(DragEdge::Left);
		}
		if vertical(self.bounds.right(), self.bounds).contains(pos) {
			return Some(DragEdge::Right);
		}
		if horizontal(self.bounds.top(), self.bounds).contains(pos) {
			return Some(DragEdge::Top);
		}
		if horizontal(self.bounds.bottom(), self.bounds).contains(pos) {
			return Some(DragEdge::Bottom);
		}
		if self.bounds.contains(pos) {
			return Some(DragEdge::Move);
		}
		None
	}
}

#[derive(Copy, Clone, PartialEq)]
struct DragState {
	edge: DragEdge,
	start: Vec2i,
	index: usize,
}

struct FlexPanelDemo {
	panels: Vec<PanelInstance>,
	drag_state: Option<DragState>,
	cursor: Vec2i,
	line_shader: shade::ShaderProgram,
	// size: Vec2i,
}

fn flex_panel(min: f32, max: f32, template: &[d2::layout::Unit]) -> Vec<f32> {
	let mut spans = vec![[0.0; 2]; template.len()];
	d2::layout::flex1d_slice(min, max, None, d2::layout::Justify::Start, template, &mut spans);
	let mut pos = Vec::with_capacity(spans.len() + 1);
	pos.push(min);
	for span in spans {
		pos.push(span[1]);
	}
	return pos;
}

impl FlexPanelDemo {
	fn new(g: &mut shade::Graphics, size: winit::dpi::PhysicalSize<u32>) -> FlexPanelDemo {
		let mut panels = Vec::new();

		let mut shader_interface = shade::shader_interface! {
			files {
				"textured.glsl" => shade::shaders::TEXTURED,
				"pixelart.glsl" => shade::shaders::PIXELART,
				"color.glsl" => shade::shaders::COLOR,
			}
		};

		{
			use d2::layout::Unit::*;

			let texture = shade::image::ImageRGBA::load_file("examples/textures/panel.png").unwrap().map_colors(|[r, g, b, a]| shade::color::Srgba8 { r, g, b, a });
			let texture = g.image(&texture);
			let shader = g.shader_compile(&mut shader_interface, "textured.glsl", &[]).unwrap();

			const W: f32 = 512.0;
			const H: f32 = 512.0;

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0, 68.0 / W, 172.0 / W, 340.0 / W, 438.0 / W, 512.0 / W],
				uv_y: &[0.0, 62.0 / H, 170.0 / H, 328.0 / H, 434.0 / H, 512.0 / H],
				template_x: &[Abs(68.0), Fr(104.0), Abs(168.0), Fr(98.0), Abs(74.0)],
				template_y: &[Abs(62.0), Fr(108.0), Abs(158.0), Fr(106.0), Abs(78.0)],
				bounds: Bounds2(Point2(50, 50), Point2(600, 400)),
				min_size: Vec2(68 + 168 + 74, 62 + 158 + 78),
			})
		}

		{
			use d2::layout::Unit::*;

			let texture = shade::image::ImageRGBA::load_file("examples/textures/panels.png").unwrap().map_colors(|[r, g, b, a]| shade::color::Srgba8 { r, g, b, a });
			let texture = g.image(&texture);
			let shader = g.shader_compile(&mut shader_interface, "pixelart.glsl", &[]).unwrap();

			const W: f32 = 64.0;
			const H: f32 = 256.0;

			const fn pos(x: i32, y: i32) -> Bounds2i {
				Bounds2i {
					mins: Vec2i { x: 128 + x * 148 - 16 * 4, y: 128 + y * 148 - 16 * 4 },
					maxs: Vec2i { x: 128 + x * 148 + 16 * 4, y: 128 + y * 148 + 16 * 4 },
				}
			}

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 16.0 / W, 20.0 / W, 32.0 / W],
				uv_y: &[0.0 / H, 12.0 / H, 18.0 / H, 32.0 / H],
				template_x: &[Abs(16.0 * 4.0), Fr(4.0 * 4.0), Abs(12.0 * 4.0)],
				template_y: &[Abs(12.0 * 4.0), Fr(6.0 * 4.0), Abs(14.0 * 4.0)],
				bounds: pos(0, 0),
				min_size: Vec2(16 + 12, 12 + 14) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 45.0 / W, 48.0 / W, 64.0 / W],
				uv_y: &[0.0 / H, 13.0 / H, 19.0 / H, 32.0 / H],
				template_x: &[Abs(13.0 * 4.0), Fr(3.0 * 4.0), Abs(16.0 * 4.0)],
				template_y: &[Abs(13.0 * 4.0), Fr(6.0 * 4.0), Abs(13.0 * 4.0)],
				bounds: pos(1, 0),
				min_size: Vec2(13 + 16, 13 + 13) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W],
				uv_y: &[32.0 / H, 38.0 / H, 58.0 / H, 64.0 / H],
				template_x: &[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)],
				template_y: &[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)],
				bounds: pos(0, 1),
				min_size: Vec2(6 + 6, 6 + 6) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 48.0 / W, 48.0 / W, 64.0 / W],
				uv_y: &[32.0 / H, 48.0 / H, 48.0 / H, 64.0 / H],
				template_x: &[Abs(16.0 * 4.0), Fr(1.0), Abs(16.0 * 4.0)],
				template_y: &[Abs(16.0 * 4.0), Fr(1.0), Abs(16.0 * 4.0)],
				bounds: pos(1, 1),
				min_size: Vec2(16 + 16, 16 + 16) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W],
				uv_y: &[64.0 / H, 75.0 / H, 88.0 / H, 96.0 / H],
				template_x: &[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)],
				template_y: &[Abs(11.0 * 4.0), Fr(13.0 * 4.0), Abs(8.0 * 4.0)],
				bounds: pos(0, 2),
				min_size: Vec2(6 + 6, 11 + 8) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 38.0 / W, 48.0 / W, 64.0 / W],
				uv_y: &[64.0 / H, 70.0 / H, 84.0 / H, 96.0 / H],
				template_x: &[Abs(6.0 * 4.0), Fr(10.0 * 4.0), Abs(14.0 * 4.0)],
				template_y: &[Abs(6.0 * 4.0), Fr(14.0 * 4.0), Abs(12.0 * 4.0)],
				bounds: pos(1, 2),
				min_size: Vec2(6 + 14, 6 + 12) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 11.0 / W, 21.0 / W, 32.0 / W],
				uv_y: &[96.0 / H, 107.0 / H, 117.0 / H, 128.0 / H],
				template_x: &[Abs(11.0 * 4.0), Fr(10.0 * 4.0), Abs(11.0 * 4.0)],
				template_y: &[Abs(11.0 * 4.0), Fr(10.0 * 4.0), Abs(11.0 * 4.0)],
				bounds: pos(0, 3),
				min_size: Vec2(11 + 11, 11 + 11) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 40.0 / W, 56.0 / W, 64.0 / W],
				uv_y: &[96.0 / H, 104.0 / H, 120.0 / H, 128.0 / H],
				template_x: &[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)],
				template_y: &[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)],
				bounds: pos(1, 3),
				min_size: Vec2(8 + 8, 8 + 8) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W],
				uv_y: &[128.0 / H, 135.0 / H, 153.0 / H, 160.0 / H],
				template_x: &[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)],
				template_y: &[Abs(7.0 * 4.0), Fr(18.0 * 4.0), Abs(7.0 * 4.0)],
				bounds: pos(2, 0),
				min_size: Vec2(6 + 6, 7 + 7) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 43.0 / W, 57.0 / W, 64.0 / W],
				uv_y: &[128.0 / H, 135.0 / H, 149.0 / H, 160.0 / H],
				template_x: &[Abs(11.0 * 4.0), Fr(14.0 * 4.0), Abs(7.0 * 4.0)],
				template_y: &[Abs(7.0 * 4.0), Fr(14.0 * 4.0), Abs(11.0 * 4.0)],
				bounds: pos(2, 1),
				min_size: Vec2(11 + 7, 7 + 11) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 8.0 / W, 24.0 / W, 32.0 / W],
				uv_y: &[160.0 / H, 169.0 / H, 183.0 / H, 192.0 / H],
				template_x: &[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)],
				template_y: &[Abs(9.0 * 4.0), Fr(14.0 * 4.0), Abs(9.0 * 4.0)],
				bounds: pos(2, 2),
				min_size: Vec2(8 + 8, 9 + 9) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 40.0 / W, 56.0 / W, 64.0 / W],
				uv_y: &[160.0 / H, 165.0 / H, 187.0 / H, 192.0 / H],
				template_x: &[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)],
				template_y: &[Abs(5.0 * 4.0), Fr(22.0 * 4.0), Abs(5.0 * 4.0)],
				bounds: pos(2, 3),
				min_size: Vec2(8 + 8, 5 + 5) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0 / W, 15.0 / W, 17.0 / W, 32.0 / W],
				uv_y: &[192.0 / H, 202.0 / H, 214.0 / H, 224.0 / H],
				template_x: &[Abs(15.0 * 4.0), Fr(2.0 * 4.0), Abs(15.0 * 4.0)],
				template_y: &[Abs(10.0 * 4.0), Fr(12.0 * 4.0), Abs(10.0 * 4.0)],
				bounds: pos(3, 0),
				min_size: Vec2(15 + 15, 10 + 10) * 4,
			});

			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[32.0 / W, 41.0 / W, 55.0 / W, 64.0 / W],
				uv_y: &[192.0 / H, 201.0 / H, 215.0 / H, 224.0 / H],
				template_x: &[Abs(9.0 * 4.0), Fr(14.0 * 4.0), Abs(9.0 * 4.0)],
				template_y: &[Abs(10.0 * 4.0), Fr(12.0 * 4.0), Abs(10.0 * 4.0)],
				bounds: pos(3, 1),
				min_size: Vec2(9 + 9, 10 + 10) * 4,
			});
		}

		let line_shader = g.shader_compile(&mut shader_interface, "color.glsl", &[]).unwrap();

		let _size = Vec2(size.width as i32, size.height as i32);

		FlexPanelDemo {
			panels,
			drag_state: None,
			cursor: Vec2i::ZERO,
			line_shader,
			// size: _size,
		}
	}
	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let mut buf = d2::DrawBuilder::<d2::TexturedVertex, d2::TexturedUniform>::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		for panel in &self.panels {
			buf.shader = panel.shader;
			buf.uniform.texture = panel.texture;

			let template_vertex = d2::TexturedTemplate { uv: Vec2::ZERO, color: Vec4(255, 255, 255, 255) };
			let pos_x = flex_panel(panel.bounds.left() as f32, panel.bounds.right() as f32, panel.template_x);
			let pos_y = flex_panel(panel.bounds.top() as f32, panel.bounds.bottom() as f32, panel.template_y);
			buf.panel_rect(&template_vertex, panel.uv_x, panel.uv_y, &pos_x, &pos_y);
		}
		buf.draw(g);

		if let Some(panel) = self.panels.last() {
			let mut buf = d2::DrawBuilder::<d2::ColorVertex, d2::ColorUniform>::new();
			buf.blend_mode = shade::BlendMode::Alpha;
			buf.cull_mode = None;
			buf.uniform.transform = Transform2::ortho(viewport.cast());
			buf.shader = self.line_shader;
			let color = Vec4(255, 255, 255, 128);
			let pen = d2::Pen {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			let pos_x = flex_panel(panel.bounds.left() as f32, panel.bounds.right() as f32, panel.template_x);
			let pos_y = flex_panel(panel.bounds.top() as f32, panel.bounds.bottom() as f32, panel.template_y);
			for x in pos_x {
				buf.draw_line(&pen, Point2(x, panel.bounds.top() as f32), Point2(x, panel.bounds.bottom() as f32));
			}
			for y in pos_y {
				buf.draw_line(&pen, Point2(panel.bounds.left() as f32, y), Point2(panel.bounds.right() as f32, y));
			}
			buf.draw(g);
		}
	}
	fn cursor_moved(&mut self, pos: Vec2i) {
		if let Some(drag_state) = &mut self.drag_state {
			let panel = &mut self.panels[drag_state.index];
			match drag_state.edge {
				DragEdge::Top => panel.bounds.mins.y = pos.y,
				DragEdge::Bottom => panel.bounds.maxs.y = pos.y,
				DragEdge::Left => panel.bounds.mins.x = pos.x,
				DragEdge::Right => panel.bounds.maxs.x = pos.x,
				DragEdge::Move => panel.bounds += pos - self.cursor,
				DragEdge::TopLeft => panel.bounds.mins = pos,
				DragEdge::TopRight => {
					panel.bounds.mins.y = pos.y;
					panel.bounds.maxs.x = pos.x;
				}
				DragEdge::BottomLeft => {
					panel.bounds.mins.x = pos.x;
					panel.bounds.maxs.y = pos.y;
				}
				DragEdge::BottomRight => panel.bounds.maxs = pos,
			}
			if panel.bounds.width() < panel.min_size.x {
				match drag_state.edge {
					DragEdge::Left | DragEdge::TopLeft | DragEdge::BottomLeft => {
						panel.bounds.mins.x = panel.bounds.maxs.x - panel.min_size.x;
					}
					DragEdge::Right | DragEdge::TopRight | DragEdge::BottomRight => {
						panel.bounds.maxs.x = panel.bounds.mins.x + panel.min_size.x;
					}
					_ => {}
				}
			}
			if panel.bounds.height() < panel.min_size.y {
				match drag_state.edge {
					DragEdge::Top | DragEdge::TopLeft | DragEdge::TopRight => {
						panel.bounds.mins.y = panel.bounds.maxs.y - panel.min_size.y;
					}
					DragEdge::Bottom | DragEdge::BottomLeft | DragEdge::BottomRight => {
						panel.bounds.maxs.y = panel.bounds.mins.y + panel.min_size.y;
					}
					_ => {}
				}
			}
		}
		self.cursor = pos;
	}
	fn hit_test(&self, pos: Vec2i) -> Option<(usize, DragEdge)> {
		for (i, panel) in self.panels.iter().enumerate().rev() {
			if let Some(edge) = panel.hit_test(pos) {
				return Some((i, edge));
			}
		}
		return None;
	}
	fn begin_drag(&mut self) -> bool {
		if let Some((index, edge)) = self.hit_test(self.cursor) {
			let last_index = self.panels.len() - 1;
			let tmp = self.panels.remove(index);
			self.panels.push(tmp);
			self.drag_state = Some(DragState { edge, start: self.cursor, index: last_index });
			return true;
		}
		false
	}
	fn end_drag(&mut self) {
		self.drag_state = None;
	}
	fn get_cursor(&self) -> Option<DragEdge> {
		let cursor = self.cursor;
		self.drag_state.as_ref().map(|s| s.edge).or_else(|| self.hit_test(cursor).map(|(_, edge)| edge))
	}
}

//----------------------------------------------------------------
// App state

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: FlexPanelDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = FlexPanelDemo::new(opengl.as_graphics(), size);
		window.window.set_title("shade flex panel demo");
		Box::new(App { window, opengl, demo })
	}
	fn sync_cursor(&mut self) {
		let cursor = match self.demo.get_cursor() {
			Some(DragEdge::Move) => winit::window::CursorIcon::Move,
			Some(DragEdge::TopLeft) | Some(DragEdge::BottomRight) => winit::window::CursorIcon::NwseResize,
			Some(DragEdge::TopRight) | Some(DragEdge::BottomLeft) => winit::window::CursorIcon::NeswResize,
			Some(DragEdge::Left) | Some(DragEdge::Right) => winit::window::CursorIcon::EwResize,
			Some(DragEdge::Top) | Some(DragEdge::Bottom) => winit::window::CursorIcon::NsResize,
			None => winit::window::CursorIcon::Default,
		};
		self.window.window.set_cursor(cursor);
	}
	fn draw(&mut self) {
		let viewport = Bounds2!(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		let g = self.opengl.as_graphics();
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.10, 0.11, 0.13, 1.0));
		self.demo.draw(g, viewport);
		g.end();
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
	let size = winit::dpi::PhysicalSize::new(960, 720);

	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, MouseButton, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

		match event {
			Event::Resumed => {
				if app.is_none() {
					let mut new_app = App::new(event_loop, size);
					new_app.sync_cursor();
					app = Some(new_app);
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.window.resize(new_size);
						app.window.window.request_redraw();
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.demo.cursor_moved(Vec2(position.x as i32, position.y as i32));
						app.sync_cursor();
						app.window.window.request_redraw();
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						match (button, state) {
							(MouseButton::Left, ElementState::Pressed) => {
								if app.demo.begin_drag() {
									app.sync_cursor();
									app.window.window.request_redraw();
								}
							}
							(MouseButton::Left, ElementState::Released) => {
								app.demo.end_drag();
								app.sync_cursor();
							}
							_ => {}
						}
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					if event.state == ElementState::Pressed && !event.repeat {
						if matches!(event.physical_key, PhysicalKey::Code(KeyCode::Escape)) {
							event_loop.exit();
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
			}
			_ => {}
		}
	});
}
