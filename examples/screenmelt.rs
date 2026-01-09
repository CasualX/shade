use std::time;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

const SCREEN_MELT_COLUMNS: i32 = 160;

struct PostProcessCopyUniforms {
	texture: shade::Texture2D,
}

impl shade::UniformVisitor for PostProcessCopyUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
	}
}

struct PostProcessMeltUniforms {
	scene: shade::Texture2D,
	delays: shade::Texture2D,
	time: f32,
}

impl shade::UniformVisitor for PostProcessMeltUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_scene", &[self.scene]);
		set.sampler2d("u_delays", &[self.delays]);
		set.value("u_time", &self.time);
	}
}

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	start_time: time::Instant,
	g: shade::gl::GlGraphics,
	pp: shade::d2::PostProcessQuad,
	pp_copy_shader: shade::Shader,
	pp_melt_shader: shade::Shader,
	gameplay_texture: shade::Texture2D,
	main_menu_texture: shade::Texture2D,
	delay_texture: shade::Texture2D,
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

		let start_time = time::Instant::now();

		let mut g = shade::gl::GlGraphics::new();

		let delay_texture = {
			let mut delays = Vec::new();
			let mut offset = 128u8;
			let mut rng = urandom::new();
			for _ in 0..SCREEN_MELT_COLUMNS {
				let a = rng.range(-1i8..=1i8) * 8;
				offset = offset.saturating_add_signed(a);
				delays.push(offset);
			}
			let info = shade::Texture2DInfo {
				format: shade::TextureFormat::R8,
				width: SCREEN_MELT_COLUMNS,
				height: 1,
				props: shade::TextureProps {
					usage: shade::TextureUsage::TEXTURE,
					mip_levels: 1,
					filter_min: shade::TextureFilter::Nearest,
					filter_mag: shade::TextureFilter::Nearest,
					wrap_u: shade::TextureWrap::Edge,
					wrap_v: shade::TextureWrap::Edge,
					border_color: [0, 0, 0, 0],
				},
			};
			g.texture2d(None, &info, &delays)
		};

		let gameplay_texture = {
			let image = shade::image::DecodedImage::load_file_gif("examples/screenmelt/e1m1.gif").unwrap();
			g.image(None, &image)
		};

		let main_menu_texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/screenmelt/main-menu.png").unwrap();
			g.image(None, &image)
		};

		let pp = shade::d2::PostProcessQuad::create(&mut g);
		let pp_copy_shader = g.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, shade::gl::shaders::POST_PROCESS_COPY_FS);
		let pp_melt_shader = g.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, shade::gl::shaders::POST_PROCESS_MELT_FS);

		Box::new(App {
			size,
			window,
			surface,
			context,
			start_time,
			g,
			gameplay_texture,
			main_menu_texture,
			delay_texture,
			pp,
			pp_copy_shader,
			pp_melt_shader,
		})
	}
	fn draw(&mut self) {
		let viewport = Bounds2::c(0, 0, self.size.width as i32, self.size.height as i32);
		self.g.begin(&shade::BeginArgs::BackBuffer { viewport });

		let elapsed = self.start_time.elapsed().as_secs_f32();

		self.pp.draw(&mut self.g, self.pp_copy_shader, shade::BlendMode::Alpha, &[&PostProcessCopyUniforms {
			texture: self.gameplay_texture,
		}]);

		self.pp.draw(&mut self.g, self.pp_melt_shader, shade::BlendMode::Alpha, &[&PostProcessMeltUniforms {
			scene: self.main_menu_texture,
			delays: self.delay_texture,
			time: (elapsed - 2.0) * 2.0,
		}]);

		self.g.end();
	}
}

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
