use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

struct PostProcessDitherUniforms {
	texture: shade::Texture2D,
	dither: shade::Texture2D,
	dither_scale: f32,
	levels: f32,
}

impl shade::UniformVisitor for PostProcessDitherUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
		set.sampler2d("u_dither", &[self.dither]);
		set.value("u_dither_scale", &self.dither_scale);
		set.value("u_levels", &self.levels);
	}
}

const POST_PROCESS_DITHER_FS: &str = r#"
#version 330 core
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
uniform sampler2D u_dither;
uniform float u_dither_scale;
uniform float u_levels;
void main() {
	ivec2 pixel_pos = ivec2(gl_FragCoord.xy);
	ivec2 dither_size = textureSize(u_dither, 0);
	ivec2 dither_pos = ivec2(floor(vec2(pixel_pos) / max(u_dither_scale, 0.0001))) % dither_size;
	float t = texelFetch(u_dither, dither_pos, 0).r;
	float threshold = (t * 255.0 + 0.5) / 256.0;
	vec4 color = texture(u_texture, v_uv);
	float levels = max(u_levels, 2.0);
	vec3 value = color.rgb * (levels - 1.0);
	vec3 quant = floor(value + threshold);
	vec3 out_rgb = clamp(quant / (levels - 1.0), 0.0, 1.0);
	frag_color = vec4(out_rgb, color.a);
}
"#;

/// Rendering state for the dither demo.
struct DitherDemo {
	pp: shade::d2::PostProcessQuad,
	pp_shader: shade::ShaderProgram,
	texture: shade::Texture2D,
	dither: [shade::Texture2D; 7],
	dither_index: usize,
	levels: f32,
}

impl DitherDemo {
	fn new(g: &mut shade::Graphics) -> DitherDemo {
		let dither1 = g.image(&shade::dither::BAYER2x2);
		let dither2 = g.image(&shade::dither::BAYER4x4);
		let dither3 = g.image(&shade::dither::BAYER8x8);
		let dither4 = g.image(&shade::dither::BAYER16x16);
		let dither5 = g.image(&shade::dither::HALFTONE16x16);
		let dither6 = g.image(&shade::dither::BNVC32x32);
		let dither7 = g.image(&shade::dither::BNVC64x64);
		let dither = [dither1, dither2, dither3, dither4, dither5, dither6, dither7];
		let texture = {
			let image = shade::image::DecodedImage::load_file("examples/screenmelt/main-menu.png").unwrap();
			g.image(&image)
		};

		let pp = shade::d2::PostProcessQuad::create(g);
		let pp_shader = g.shader_compile(shade::gl::shaders::POST_PROCESS_VS, POST_PROCESS_DITHER_FS);

		DitherDemo {
			pp,
			pp_shader,
			texture,
			dither,
			dither_index: 0,
			levels: 3.0,
		}
	}

	fn next_index(&mut self, offset: isize) {
		let len = self.dither.len() as isize;
		let next = (self.dither_index as isize + offset).rem_euclid(len);
		self.dither_index = next as usize;
	}

	fn adjust_levels(&mut self, delta: f32) {
		self.levels = (self.levels + delta).max(2.0);
	}

	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// let elapsed = self.epoch.elapsed().as_secs_f32();
		// let index = (elapsed.floor() as i32 as usize) % self.dither.len();
		let index = self.dither_index;

		self.pp.draw(g, self.pp_shader, shade::BlendMode::Alpha, &[&PostProcessDitherUniforms {
			texture: self.texture,
			dither: self.dither[index],
			dither_scale: 1.0,
			levels: self.levels,
		}]);

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

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: DitherDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = DitherDemo::new(opengl.as_graphics());
		Box::new(App { window, opengl, demo })
	}
	fn draw(&mut self) {
		let viewport = Bounds2::c(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let size = winit::dpi::PhysicalSize::new(800, 600);
	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

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
				WindowEvent::KeyboardInput { event, .. } => {
					if event.state == ElementState::Pressed && !event.repeat {
						if let Some(app) = app.as_deref_mut() {
							match event.physical_key {
								PhysicalKey::Code(KeyCode::ArrowLeft) => app.demo.next_index(-1),
								PhysicalKey::Code(KeyCode::ArrowRight) => app.demo.next_index(1),
								PhysicalKey::Code(KeyCode::ArrowUp) => app.demo.adjust_levels(1.0),
								PhysicalKey::Code(KeyCode::ArrowDown) => app.demo.adjust_levels(-1.0),
								_ => {}
							}
							app.window.window.request_redraw();
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
					app.window.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
