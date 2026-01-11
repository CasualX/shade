use std::{mem, time};
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec2f,
	uv: Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "a_pos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.position) as u16,
			},
			shade::VertexAttribute {
				name: "a_uv",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.uv) as u16,
			},
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core

out vec4 o_fragColor;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform sampler2D u_distortion;
uniform float u_time;

void main()
{
	vec2 uv = v_uv;

	// Layered distortion for organic feel
	float distortion1 = texture(u_distortion, fract(uv * 1.0 + vec2(u_time * 0.2, u_time * 0.25))).r;
	float distortion2 = texture(u_distortion, fract(uv * 2.5 + vec2(-u_time * 0.15, u_time * 0.1))).r;
	float distortion3 = texture(u_distortion, fract(uv * 0.75 + vec2(u_time * 0.05, -u_time * 0.2))).r;

	float distortion = (distortion1 * 0.5 + distortion2 * 0.3 + distortion3 * 0.2) - 0.5;

	uv += vec2(distortion * 0.1, distortion * 0.15);

	// Main wave layer
	float v = texture(u_texture, uv * vec2(4.0, 4.0)).r;
	vec3 mainColor = mix(vec3(0.0, 0.0, 0.5), vec3(0.5, 0.8, 1.0), v);

	// Second darker shadow layer
	float u = texture(u_texture, uv * vec2(2.0, 2.0) + vec2(0.3, 0.3)).r;
	vec3 shadowColor = mix(vec3(0.0, 0.0, 0.0), vec3(0.1, 0.2, 0.3), u);

	// Composite: shadow appears under water
	vec3 finalColor = mainColor - shadowColor * 0.5;

	o_fragColor = vec4(finalColor, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core

in vec2 a_pos;
in vec2 a_uv;

out vec2 v_uv;

void main()
{
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

#[derive(Clone, Debug)]
struct Uniform {
	time: f32,
	texture: shade::Texture2D,
	distortion: shade::Texture2D,
}

impl shade::UniformVisitor for Uniform {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
		set.value("u_texture", &self.texture);
		set.value("u_distortion", &self.distortion);
	}
}

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

struct ZeldaWaterDemo {
	g: shade::gl::GlGraphics,
	vb: shade::VertexBuffer,
	ib: shade::IndexBuffer,
	texture: shade::Texture2D,
	distortion: shade::Texture2D,
	shader: shade::Shader,
	start_time: time::Instant,
}

impl ZeldaWaterDemo {
	fn new() -> ZeldaWaterDemo {
		let mut g = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: false });

		let vb = g.vertex_buffer(
			None,
			&[
				Vertex { position: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
				Vertex { position: Vec2f(1.0, -1.0), uv: Vec2f(1.0, 0.0) },
				Vertex { position: Vec2f(-1.0, 1.0), uv: Vec2f(0.0, 1.0) },
				Vertex { position: Vec2f(1.0, 1.0), uv: Vec2f(1.0, 1.0) },
			],
			shade::BufferUsage::Static,
		);

		let ib = g.index_buffer(None, &[0u16, 1, 2, 1, 3, 2], 4, shade::BufferUsage::Static);

		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/zeldawater/water.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			};
			g.image(Some("font"), &(&image, &props))
		};

		let distortion = {
			let image = shade::image::DecodedImage::load_file_png("examples/zeldawater/distort.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			};
			g.image(None, &(&image, &props))
		};

		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);
		let start_time = time::Instant::now();

		ZeldaWaterDemo { g, vb, ib, texture, distortion, shader, start_time }
	}

	fn draw(&mut self, window: &GlWindow) {
		let viewport = Bounds2::c(0, 0, window.size.width as i32, window.size.height as i32);
		self.g.begin(&shade::BeginArgs::BackBuffer { viewport });

		shade::clear!(self.g, color: Vec4(0.2, 0.5, 0.2, 1.0));

		let time = self.start_time.elapsed().as_secs_f32();
		let uniform = Uniform { time, texture: self.texture, distortion: self.distortion };

		self.g.draw_indexed(&shade::DrawIndexedArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vb,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: self.ib,
			index_start: 0,
			index_end: 6,
			uniforms: &[&uniform],
			instances: -1,
		});

		self.g.end();
	}
}

struct App {
	window: GlWindow,
	demo: ZeldaWaterDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let demo = ZeldaWaterDemo::new();
		Box::new(App { window, demo })
	}
	fn draw(&mut self) {
		self.demo.draw(&self.window);
	}
}

//----------------------------------------------------------------

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
