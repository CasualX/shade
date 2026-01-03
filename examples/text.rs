use shade::d2;
use shade::cvmath::*;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;

//----------------------------------------------------------------
// Application state

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	font: d2::FontResource<shade::msdfgen::Font>,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Box<App> {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let size = winit::dpi::PhysicalSize::new(800, 600);

		let template = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_inner_size(size);

		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template, |configs| configs.max_by_key(|c| c.num_samples()).unwrap())
			.expect("Failed to build window and GL config");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window
			.window_handle()
			.expect("Failed to get raw window handle")
			.as_raw();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe {
			gl_display.create_context(&gl_config, &context_attributes)
		}.expect("Failed to create GL context");

		let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe {
			gl_display.create_window_surface(&gl_config, &attrs)
		}.expect("Failed to create GL surface");

		let context = not_current
			.make_current(&surface)
			.expect("Failed to make GL context current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		// Create the graphics context
		let mut g = shade::gl::GlGraphics::new();

		let font = {
			// Parse the font metadata
			let font: shade::msdfgen::FontDto = serde_json::from_str(include_str!("font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();

			// Load the texture
			let texture = {
				let image = shade::image::DecodedImage::load_file_png("examples/font/font.png").unwrap();
				g.image(Some("font"), &image)
			};

			// Compile the shader
			let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);

			d2::FontResource { font, texture, shader }
		};

		Box::new(App { size, window, surface, context, g, font })
	}

	fn draw(&mut self) {
		// Render the frame
		let viewport = Bounds2::c(0, 0, self.size.width as i32, self.size.height as i32);
		self.g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		// Clear the screen
		self.g.clear(&shade::ClearArgs {
			color: Some(Vec4(0.4, 0.4, 0.7, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		let mut cv = d2::TextBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.uniform.transform = Transform2::ortho(Bounds2::c(0.0, 0.0, self.size.width as f32, self.size.height as f32));
		cv.uniform.outline_width_relative = 0.125;

		let mut pos = Vec2(0.0, 0.0);
		let mut scribe = d2::Scribe {
			font_size: 64.0,
			line_height: 64.0 * 1.5,
			x_pos: pos.x,
			top_skew: 8.0,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);

		cv.text_write(&self.font, &mut scribe, &mut pos, "Hello, \x1b[font_size=96.0]\x1b[font_width_scale=1.5]\x1b[top_skew=0.0]world!");

		scribe.font_size = 32.0;
		scribe.line_height = 32.0;
		scribe.font_width_scale = 1.0;
		scribe.color = Vec4(255, 255, 0, 255);

		let bounds = Bounds2::c(0.0, 0.0, self.size.width as f32, self.size.height as f32);
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleCenter, "These\nare\nmultiple\nlines.\n");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleLeft, "[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleRight, "↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶\n△▽◁▷\n☐☑☒🗹🗷\n⏰💎🔹⚡⛔🏁");

		scribe.top_skew = 8.0;
		let rainbow = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W";
		let rainbow_width = scribe.text_width(&mut {Vec2::ZERO}, &self.font.font, rainbow);
		let mut pos = Vec2f((self.size.width as f32 - rainbow_width) * 0.5, self.size.height as f32 - scribe.font_size);
		cv.text_write(&self.font, &mut scribe, &mut pos, rainbow);

		cv.draw(&mut self.g);

		// Finish rendering
		self.g.end();
	}
}

//----------------------------------------------------------------

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{Event, WindowEvent};

		match event {
			Event::Resumed => {
				if app.is_none() {
					app = Some(App::new(event_loop));
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
						let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
						app.size = new_size;
						app.surface.resize(&app.context, width, height);
					}
				}
				WindowEvent::CloseRequested => event_loop.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						app.draw();
						app.surface.swap_buffers(&app.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					app.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
