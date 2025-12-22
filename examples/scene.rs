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
			shade::VertexAttribute::with::<Vec3f>("aPos", dataview::offset_of!(MyVertex3.position)),
			shade::VertexAttribute::with::<Vec2f>("aTexCoord", dataview::offset_of!(MyVertex3.tex_coord)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("aColor", dataview::offset_of!(MyVertex3.color)),
		],
	};
}

//----------------------------------------------------------------
// Shader and uniforms

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec4 VertexColor;
in vec2 TexCoord;

uniform sampler2D tex;

void main() {
	ivec2 texSize = textureSize(tex, 0);
	vec4 color = texture(tex, TexCoord / texSize) * VertexColor;
	if (color.a < 0.5) {
		discard;
	}
	color.a = 1.0;
	FragColor = color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec4 aColor;

out vec4 VertexColor;
out vec2 TexCoord;

uniform mat4x4 transform;

void main()
{
	VertexColor = aColor;
	TexCoord = aTexCoord;
	gl_Position = transform * vec4(aPos, 1.0);
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
		set.value("transform", &self.transform);
		set.value("tex", &self.texture);
	}
}

//----------------------------------------------------------------
// Application state

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	texture: shade::Texture2D,
	shader: shade::Shader,
	time_base: time::Instant,
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

		// Load the texture
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/scene tiles.png").unwrap();
			let props = shade::TextureProps {
				filter_min: shade::TextureFilter::Nearest,
				filter_mag: shade::TextureFilter::Nearest,
				wrap_u: shade::TextureWrap::ClampEdge,
				wrap_v: shade::TextureWrap::ClampEdge,
			};
			g.image(Some("scene tiles"), &(&image, &props))
		};

		// Create the shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		let time_base = time::Instant::now();

		Box::new(App { size, window, surface, context, g, texture, shader, time_base })
	}

	fn draw(&mut self) {
		let app = self;
		let size = app.size;
		let curtime = time::Instant::now().duration_since(app.time_base).as_secs_f32();

		// Render the frame
		app.g.begin();

		// Clear the screen
		app.g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		// Update the camera
		let projection = Mat4::perspective(Angle::deg(45.0), size.width as f32 / size.height as f32, 0.1, 1000.0, (Hand::RH, Clip::NO));
		let view = {
			let eye = Vec3(32.0 + (curtime * 2.0).sin() * 32.0, 100.0 + (curtime * 1.5).sin() * 32.0, -100.0) * 1.5;
			let target = Vec3(96.0 * 0.5, 0.0, 32.0);
			let up = Vec3(0.0, 1.0, 0.0);
			Transform3f::look_at(eye, target, up, Hand::RH)
		};
		let transform = projection * view;

		let mut cv = shade::im::DrawBuilder::<MyVertex3, MyUniform3>::new();
		cv.viewport = Bounds2::c(0, 0, size.width as i32, size.height as i32);
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.depth_test = Some(shade::DepthTest::Less);
		cv.shader = app.shader;
		cv.uniform.transform = transform;
		cv.uniform.texture = app.texture;
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

		cv.draw(&mut app.g, shade::Surface::BACK_BUFFER);

		// Finish the frame
		app.g.end();
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
