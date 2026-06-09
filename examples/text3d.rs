use shade::cvmath::*;
use shade::{d2, d3};
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;

//----------------------------------------------------------------
// Application state

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

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_inner_size(size)
			.with_title("shade text3d");

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

		let surface_attributes_builder =
			SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(Some(true));
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

		GlWindow { size, window, surface, context }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
	}
}

struct Text3dDemo {
	font: d2::FontResource<shade::msdfgen::Font>,
	camera: d3::ArcballCamera,
	axes: d3::axes::AxesModel,
}

impl Text3dDemo {
	fn new(g: &mut shade::Graphics) -> Text3dDemo {
		let mut shader_source = shade::shader_interface! {
			files {
				"mtsdf.glsl" => include_str!("../src/shaders/mtsdf.glsl"),
				"color3d.glsl" => include_str!("../src/shaders/color3d.glsl"),
			}
		};

		let font = {
			let font: shade::msdfgen::FontDto = serde_json::from_str(include_str!("font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();

			let texture = {
				let image = shade::image::DecodedImage::load_file_png("examples/font/font.png").unwrap()
					.to_rgba().map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
				g.image(&image)
			};

			let shader = g.shader_compile(&mut shader_source, "mtsdf.glsl", &[
				shade::ShaderDefine { name: "MTSDF_3D", value: None },
			]);

			d2::FontResource { font, texture, shader }
		};

		let color3d_shader = g.shader_compile(&mut shader_source, "color3d.glsl", &[]);
		let axes = d3::axes::AxesModel::create(g, color3d_shader);
		let camera = d3::ArcballCamera::new(Vec3f(0.0, -8.0, 4.6), Vec3f(0.0, -0.4, 0.9), Vec3f::Z);

		Text3dDemo { font, camera, axes }
	}

	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.08, 0.10, 0.12, 1.0), depth: 1.0);

		let camera = self.camera(viewport);
		self.axes.draw(g, &camera, &d3::axes::AxesInstance {
			local: Transform3f::scaling(Vec3f::dup(3.0)),
			depth_test: Some(shade::Compare::LessEqual),
		});
		self.draw_text(g, &camera);
		g.end();
	}

	fn camera(&self, viewport: Bounds2i) -> d3::Camera {
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let position = self.camera.position();
		let hand = Hand::RH;
		let view = self.camera.view_matrix(hand);
		let clip = Clip::NO;
		let (near, far) = (0.01, 200.0);
		let projection = Mat4::perspective(Angle::deg(55.0), aspect_ratio, near, far, (hand, clip));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
	}

	fn draw_text(&self, g: &mut shade::Graphics, camera: &d3::Camera) {
		let mut buf = d2::TextBuffer3::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.depth_test = Some(shade::Compare::LessEqual);
		buf.cull_mode = None;
		buf.uniform.camera_transform = camera.view_proj;
		buf.uniform.text.outline_width_relative = 0.10;

		let mut scribe = d2::Scribe {
			font_width_scale: 1.0,
			letter_spacing: -1.5,
			..Default::default()
		};

		scribe.font_size = 52.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(255, 255, 255, 255);
		scribe.outline = Vec4(13, 24, 36, 255);
		scribe.set_baseline_relative(0.5);
		let front = Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(-2.7, 0.6, 1.6));
		self.add_text_plane(g, &mut buf, "front plane", 0.018, front, &scribe);

		scribe.font_size = 48.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(255, 220, 130, 255);
		scribe.outline = Vec4(20, 14, 4, 255);
		scribe.set_baseline_relative(0.5);
		let floor = Transform3f::compose(Vec3f::X, Vec3f::Y, Vec3f::Z, Vec3f(-2.8, -2.3, 0.0));
		self.add_text_plane(g, &mut buf, "on the floor", 0.018, floor, &scribe);

		scribe.font_size = 44.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(140, 220, 255, 255);
		scribe.outline = Vec4(4, 18, 28, 255);
		scribe.set_baseline_relative(0.5);
		let wall = Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(-3.0, -1.4, 0.9));
		self.add_text_plane(g, &mut buf, "side wall", 0.017, wall, &scribe);

		scribe.font_size = 46.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(180, 255, 185, 255);
		scribe.outline = Vec4(2, 26, 11, 255);
		scribe.set_baseline_relative(0.5);
		let tilted = Transform3f::translation(Vec3f(1.2, -0.6, 1.4))
			* Transform3f::rotation(Vec3f::Z, Angle::deg(-28.0))
			* Transform3f::rotation(Vec3f::X, Angle::deg(62.0));
		self.add_text_plane(g, &mut buf, "tilted placard", 0.017, tilted, &scribe);

		scribe.font_size = 32.0;
		scribe.line_height = 36.0;
		scribe.color = Vec4(255, 240, 120, 255);
		scribe.outline = Vec4(38, 26, 0, 255);
		scribe.top_skew = 0.0;
		scribe.set_baseline_relative(0.5);
		let glyph_wall = Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(3.2, -0.5, 1.5));
		self.add_text_plane(g, &mut buf, "↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶", 0.012, glyph_wall, &scribe);

		scribe.font_size = 28.0;
		scribe.line_height = 31.0;
		scribe.color = Vec4(210, 225, 255, 255);
		scribe.outline = Vec4(10, 20, 42, 255);
		scribe.set_baseline_relative(0.5);
		let status_board = Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(0.5, 2.8, 1.5));
		let text = "[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness\n☐☑☒  🗹🗷";
		self.add_text_plane(g, &mut buf, text, 0.013, status_board, &scribe);

		scribe.font_size = 30.0;
		scribe.line_height = 34.0;
		scribe.color = Vec4(255, 255, 255, 255);
		scribe.outline = Vec4(28, 10, 16, 255);
		scribe.top_skew = 8.0;
		scribe.set_baseline_relative(0.5);
		let rainbow = Transform3f::translation(Vec3f(2.6, 1.0, 2.4))
			* Transform3f::rotation(Vec3f::Y, Angle::deg(-25.0))
			* Transform3f::rotation(Vec3f::X, Angle::deg(20.0));
		let text = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W\n⏰💎🔹⚡⛔🏁";
		self.add_text_plane(g, &mut buf, text, 0.014, rainbow, &scribe);

		scribe.font_size = 26.0;
		scribe.line_height = 29.0;
		scribe.top_skew = 0.0;
		scribe.color = Vec4(190, 255, 220, 255);
		scribe.outline = Vec4(5, 32, 18, 255);
		scribe.set_baseline_relative(0.5);
		let corner_notes = Transform3f::translation(Vec3f(-0.4, -3.0, 1.2))
			* Transform3f::rotation(Vec3f::X, Angle::deg(86.0))
			* Transform3f::rotation(Vec3f::Z, Angle::deg(14.0));
		let text = "△▽◁▷\ntext in orbit\nshade glyph picnic";
		self.add_text_plane(g, &mut buf, text, 0.012, corner_notes, &scribe);
	}

	fn add_text_plane(&self, g: &mut shade::Graphics, buf: &mut d2::TextBuffer3, text: &str, scale: f32, plane: Transform3f, scribe: &d2::Scribe) {
		buf.uniform.plane_transform = plane;
		buf.uniform.text.transform = Transform2f::compose(Vec2f(scale, 0.0), Vec2f(0.0, -scale), Vec2f::ZERO);

		let bounds = Bounds2f::point(Vec2f::ZERO, Vec2f::ZERO);
		buf.text_box(&self.font, scribe, &bounds, d2::TextAlign::MiddleCenter, text);
		buf.draw(g);
	}
}

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: Text3dDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = Text3dDemo::new(opengl.as_graphics());
		Box::new(App { window, opengl, demo })
	}
	fn draw(&mut self) {
		let viewport = Bounds2!(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
	}
}

//----------------------------------------------------------------

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().unwrap();
	let size = winit::dpi::PhysicalSize::new(960, 720);

	let mut app: Option<Box<App>> = None;
	let mut left_click = false;
	let mut middle_click = false;
	let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

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
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.window.resize(new_size);
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let dx = position.x as f32 - cursor_position.x as f32;
						let dy = position.y as f32 - cursor_position.y as f32;
						if left_click {
							app.demo.camera.rotate(-dx, -dy);
						}
						if middle_click {
							app.demo.camera.zoom(dy * 0.01);
						}
					}
					cursor_position = position;
				}
				WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
					left_click = matches!(state, ElementState::Pressed);
				}
				WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
					middle_click = matches!(state, ElementState::Pressed);
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
					app.window.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
