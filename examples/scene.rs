use std::mem;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::time;

use glutin::prelude::*;
use shade::cvmath::*;

//----------------------------------------------------------------
// Vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MyVertex3 {
	position: Vec3f,
	tex_coord: Vec2f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for MyVertex3 {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<MyVertex3>() as u16,
		alignment: mem::align_of::<MyVertex3>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(MyVertex3.position)),
			shade::VertexAttribute::with::<Vec2f>("a_uv", dataview::offset_of!(MyVertex3.tex_coord)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(MyVertex3.color)),
		],
	};
}

//----------------------------------------------------------------
// Shader and uniforms

const FRAGMENT_SHADER: &str = r#"
#version 330 core

out vec4 o_fragColor;

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	ivec2 texSize = textureSize(u_texture, 0);
	vec4 color = texture(u_texture, v_uv / texSize) * v_color;
	if (color.a < 0.5) {
		discard;
	}
	color.a = 1.0;
	o_fragColor = color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
in vec3 a_pos;
in vec2 a_uv;
in vec4 a_color;

out vec4 v_color;
out vec2 v_uv;

uniform mat4x4 u_transform;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_color = srgbToLinear(a_color);
	v_uv = a_uv;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

#[derive(Clone, Debug, PartialEq)]
struct MyUniform3 {
	transform: Mat4f,
	texture: shade::Texture2D,
}

impl Default for MyUniform3 {
	fn default() -> Self {
		MyUniform3 {
			transform: Mat4::IDENTITY,
			texture: shade::Texture2D::INVALID,
		}
	}
}

impl shade::UniformVisitor for MyUniform3 {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
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

struct SceneDemo {
	texture: shade::Texture2D,
	shader: shade::Shader,
	epoch: time::Instant,
}

impl SceneDemo {
	fn new(g: &mut shade::Graphics) -> SceneDemo {
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/scene tiles.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Nearest,
				filter_mag: shade::TextureFilter::Nearest,
				wrap_u: shade::TextureWrap::Edge,
				wrap_v: shade::TextureWrap::Edge,
				..Default::default()
			};
			g.image(Some("scene tiles"), &(&image, &props))
		};

		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);
		let epoch = time::Instant::now();

		SceneDemo { texture, shader, epoch }
	}

	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let curtime = self.epoch.elapsed().as_secs_f32();

		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		shade::clear!(g, color: Vec4(0.2, 0.2, 0.5, 1.0), depth: 1.0);

		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let projection = Mat4::perspective(
			Angle::deg(45.0),
			aspect_ratio,
			0.1,
			1000.0,
			(Hand::RH, Clip::NO),
		);
		let view = {
			let eye = Vec3(32.0 + (curtime * 2.0).sin() * 32.0, 100.0 + (curtime * 1.5).sin() * 32.0, -100.0) * 1.5;
			let target = Vec3(96.0 * 0.5, 0.0, 32.0);
			let up = Vec3(0.0, 1.0, 0.0);
			Transform3f::look_at(eye, target, up, Hand::RH)
		};
		let transform = projection * view;

		let mut cv = shade::im::DrawBuilder::<MyVertex3, MyUniform3>::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.depth_test = Some(shade::Compare::Less);
		cv.shader = self.shader;
		cv.uniform.transform = transform;
		cv.uniform.texture = self.texture;
		floor_tile(&mut cv, 0, 0, &GRASS);
		floor_tile(&mut cv, 1, 0, &GRASS);
		floor_tile(&mut cv, 2, 0, &GRASS);
		floor_tile(&mut cv, 0, 1, &GRASS);
		floor_tile(&mut cv, 1, 1, &GRASS);
		floor_tile(&mut cv, 2, 1, &GRASS);
		floor_thing(&mut cv, 0, 0, &DROP);
		floor_thing(&mut cv, 1, 1, &DROP);
		floor_thing(&mut cv, 2, 0, &DROP);
		floor_thing(&mut cv, 1, 0, &BEAR);
		cv.draw(g);

		g.end();
	}
}

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: SceneDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = SceneDemo::new(opengl.as_graphics());
		Box::new(App { window, opengl, demo })
	}
	fn draw(&mut self) {
		let viewport = Bounds2::c(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
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

struct Sprite {
	left: f32,
	up: f32,
	right: f32,
	down: f32,
}

const GRASS: Sprite = Sprite { left: 0.0 * 34.0 + 1.0, up: 1.0 * 34.0 + 1.0, right: 0.0 * 34.0 + 32.0, down: 1.0 * 34.0 + 32.0 };
const DROP: Sprite = Sprite { left: 1.0 * 34.0 + 1.0, up: 1.0 * 34.0 + 1.0, right: 1.0 * 34.0 + 32.0, down: 1.0 * 34.0 + 32.0 };
const BEAR: Sprite = Sprite { left: 3.0, up: 68.0, right: 49.0, down: 152.0 };

fn floor_tile(cv: &mut shade::im::DrawBuilder<MyVertex3, MyUniform3>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: Vec2(sprite.left, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: Vec2(sprite.left, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3((x + 1) as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: Vec2(sprite.right, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3((x + 1) as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: Vec2(sprite.right, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
}

fn floor_thing(cv: &mut shade::im::DrawBuilder<MyVertex3, MyUniform3>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	let yoffs = -7.0;
	let zoffs1 = 12.0;
	let zoffs2 = 24.0;
	let width = sprite.right - sprite.left;
	let height = sprite.down - sprite.up;
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, 0.0 + yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: Vec2(sprite.left, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: Vec2(sprite.left, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0 + width, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: Vec2(sprite.right, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0 + width, 0.0 + yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: Vec2(sprite.right, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
}
