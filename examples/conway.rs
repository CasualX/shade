use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

// Simulation grid size (in cells / texels)
const FIELD_WIDTH: i32 = 256;
const FIELD_HEIGHT: i32 = 256;

const CONWAY_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

uniform sampler2D u_state;

int alive_at(ivec2 p, ivec2 size) {
	ivec2 q = ivec2(
		(p.x % size.x + size.x) % size.x,
		(p.y % size.y + size.y) % size.y
	);
	return (texelFetch(u_state, q, 0).r > 0.5) ? 1 : 0;
}

void main() {
	ivec2 size = textureSize(u_state, 0);
	ivec2 p = ivec2(gl_FragCoord.xy);

	int n = 0;
	n += alive_at(p + ivec2(-1, -1), size);
	n += alive_at(p + ivec2( 0, -1), size);
	n += alive_at(p + ivec2( 1, -1), size);
	n += alive_at(p + ivec2(-1,  0), size);
	n += alive_at(p + ivec2( 1,  0), size);
	n += alive_at(p + ivec2(-1,  1), size);
	n += alive_at(p + ivec2( 0,  1), size);
	n += alive_at(p + ivec2( 1,  1), size);

	int alive = alive_at(p, size);
	int next_alive = 0;
	if (alive == 1) {
		next_alive = ((n == 2) || (n == 3)) ? 1 : 0;
	}
	else {
		next_alive = (n == 3) ? 1 : 0;
	}

	float v = float(next_alive);
	o_fragColor = vec4(v, 0.0, 0.0, 1.0);
}
"#;

const DISPLAY_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;
in vec2 v_uv;

uniform sampler2D u_state;

void main() {
	float v = texture(u_state, v_uv).r;
	o_fragColor = vec4(vec3(v), 1.0);
}
"#;

struct StateUniforms {
	state: shade::Texture2D,
}

impl shade::UniformVisitor for StateUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_state", &[self.state]);
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

struct ConwayDemo {
	g: shade::gl::GlGraphics,
	pp: shade::d2::PostProcessQuad,
	conway_shader: shade::Shader,
	display_shader: shade::Shader,
	field_size: Vec2i,
	ping: usize,
	state: [shade::Texture2D; 2],
}

impl ConwayDemo {
	fn new() -> ConwayDemo {
		let mut g = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: false });

		let pp = shade::d2::PostProcessQuad::create(&mut g);
		let conway_shader = g.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, CONWAY_FS);
		let display_shader = g.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, DISPLAY_FS);

		let field_size = Vec2::new(FIELD_WIDTH.max(1), FIELD_HEIGHT.max(1));
		let seed = seed_data(field_size.x, field_size.y);
		let info = shade::Texture2DInfo {
			format: shade::TextureFormat::R8,
			width: field_size.x,
			height: field_size.y,
			props: shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage!(WRITE | SAMPLED | COLOR_TARGET),
				filter_min: shade::TextureFilter::Nearest,
				filter_mag: shade::TextureFilter::Nearest,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			},
		};

		let state0 = g.texture2d(Some("conway_state0"), &info, &seed);
		let state1 = g.texture2d_create(Some("conway_state1"), &info);

		ConwayDemo {
			g,
			pp,
			conway_shader,
			display_shader,
			field_size,
			ping: 0,
			state: [state0, state1],
		}
	}

	fn step(&mut self) {
		let src = self.state[self.ping];
		let dst = self.state[1 - self.ping];

		let viewport = Bounds2::c(0, 0, self.field_size.x, self.field_size.y);
		self.g.begin(&shade::BeginArgs::Immediate {
			viewport,
			color: &[dst],
			levels: None,
			depth: shade::Texture2D::INVALID,
		});
		self.pp.draw(
			&mut self.g,
			self.conway_shader,
			shade::BlendMode::Solid,
			&[&StateUniforms { state: src }],
		);
		self.g.end();

		self.ping = 1 - self.ping;
	}

	fn draw(&mut self, window: &GlWindow) {
		self.step();

		let viewport = Bounds2::c(0, 0, window.size.width as i32, window.size.height as i32);
		self.g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(self.g, color: Vec4(0.0, 0.0, 0.0, 1.0));
		self.pp.draw(
			&mut self.g,
			self.display_shader,
			shade::BlendMode::Solid,
			&[&StateUniforms {
				state: self.state[self.ping],
			}],
		);
		self.g.end();
	}
}

fn seed_data(width: i32, height: i32) -> Vec<u8> {
	let width = width.max(1);
	let height = height.max(1);
	let mut rng = urandom::new();

	let n = (width as usize) * (height as usize);
	let mut data = vec![0u8; n];

	let x0 = width / 4;
	let x1 = (width * 3) / 4;
	let y0 = height / 4;
	let y1 = (height * 3) / 4;

	for y in y0..y1 {
		for x in x0..x1 {
			let i = (y as usize) * (width as usize) + (x as usize);
			// ~20% alive; stored as 0 or 255 in R8.
			let r = rng.range(0u8..=9u8);
			data[i] = if r < 2 { 255 } else { 0 };
		}
	}

	data
}

struct App {
	window: GlWindow,
	demo: ConwayDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let demo = ConwayDemo::new();
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
