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

/// OpenGL Window wrapper.
struct GlWindow {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
}

impl GlWindow {
	fn new(
		event_loop: &winit::event_loop::ActiveEventLoop,
		size: winit::dpi::PhysicalSize<u32>,
	) -> GlWindow {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
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

/// Rendering state for the screen-melt demo.
struct ScreenMeltDemo {
	epoch: time::Instant,
	graphics: shade::gl::GlGraphics,
	pp: shade::d2::PostProcessQuad,
	pp_copy_shader: shade::Shader,
	pp_melt_shader: shade::Shader,
	gameplay_texture: shade::Texture2D,
	main_menu_texture: shade::Texture2D,
	delay_texture: shade::Texture2D,
}

impl ScreenMeltDemo {
	fn new() -> ScreenMeltDemo {
		let epoch = time::Instant::now();

		let mut graphics = shade::gl::GlGraphics::new(shade::gl::GlConfig {
			srgb: false,
		});

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
					..Default::default()
				},
			};
			graphics.texture2d(None, &info, &delays)
		};

		let gameplay_texture = {
			let image = shade::image::DecodedImage::load_file_gif("examples/screenmelt/e1m1.gif").unwrap();
			graphics.image(None, &image)
		};

		let main_menu_texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/screenmelt/main-menu.png").unwrap();
			graphics.image(None, &image)
		};

		let pp = shade::d2::PostProcessQuad::create(&mut graphics);
		let pp_copy_shader = graphics.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, shade::gl::shaders::POST_PROCESS_COPY_FS);
		let pp_melt_shader = graphics.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, shade::gl::shaders::POST_PROCESS_MELT_FS);

		ScreenMeltDemo {
			epoch,
			graphics,
			gameplay_texture,
			main_menu_texture,
			delay_texture,
			pp,
			pp_copy_shader,
			pp_melt_shader,
		}
	}

	fn draw(&mut self, window: &GlWindow) {
		let viewport = Bounds2::c(0, 0, window.size.width as i32, window.size.height as i32);
		self.graphics.begin(&shade::BeginArgs::BackBuffer { viewport });

		let elapsed = self.epoch.elapsed().as_secs_f32();

		self.pp.draw(&mut self.graphics, self.pp_copy_shader, shade::BlendMode::Alpha, &[&PostProcessCopyUniforms {
			texture: self.gameplay_texture,
		}]);

		self.pp.draw(&mut self.graphics, self.pp_melt_shader, shade::BlendMode::Alpha, &[&PostProcessMeltUniforms {
			scene: self.main_menu_texture,
			delays: self.delay_texture,
			time: (elapsed - 2.0) * 2.0,
		}]);

		self.graphics.end();
	}
}

struct App {
	window: GlWindow,
	demo: ScreenMeltDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let demo = ScreenMeltDemo::new();
		Box::new(App { window, demo })
	}
	fn draw(&mut self) {
		self.demo.draw(&self.window);
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let size = winit::dpi::PhysicalSize::new(800, 600);
	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{Event, WindowEvent};

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
