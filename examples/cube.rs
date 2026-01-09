use std::mem;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

//----------------------------------------------------------------
// Geometry, uniforms and shader

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct CubeVertex {
	position: Vec3f,
	tex_coord: Vec2f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for CubeVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<CubeVertex>() as u16,
		alignment: mem::align_of::<CubeVertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(CubeVertex.position)),
			shade::VertexAttribute::with::<Vec2f>("a_uv", dataview::offset_of!(CubeVertex.tex_coord)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(CubeVertex.color)),
		],
	};
}

const X_MIN: f32 = -1.0;
const X_MAX: f32 =  1.0;
const Y_MIN: f32 = -1.0;
const Y_MAX: f32 =  1.0;
const Z_MIN: f32 = -1.0;
const Z_MAX: f32 =  1.0;

static VERTICES: [CubeVertex; 24] = [
	// Front face
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([255,   0,   0, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([192,   0,   0, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([192,   0,   0, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([128,   0,   0, 255]) },
	// Back face
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([  0, 255, 255, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([  0, 192, 192, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([  0, 192, 192, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([  0, 128, 128, 255]) },
	// Left face
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([  0, 255,   0, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([  0, 192,   0, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([  0, 192,   0, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([  0, 128,   0, 255]) },
	// Right face
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([255,   0, 255, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([192,   0, 192, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([192,   0, 192, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([128,   0, 128, 255]) },
	// Top face
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([  0,   0, 255, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([  0,   0, 192, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([  0,   0, 192, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([  0,   0, 128, 255]) },
	// Bottom face
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: Vec2(0.0, 0.0), color: shade::norm!([255, 255, 255, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: Vec2(1.0, 0.0), color: shade::norm!([192, 192, 192, 255]) },
	CubeVertex { position: Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: Vec2(0.0, 1.0), color: shade::norm!([192, 192, 192, 255]) },
	CubeVertex { position: Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: Vec2(1.0, 1.0), color: shade::norm!([128, 128, 128, 255]) },
];

static INDICES: [u8; 36] = [
	 0, 1, 2,  2, 1, 3, // front
	 4, 5, 6,  6, 5, 7, // back
	 8, 9,10, 10, 9,11, // left
	12,13,14, 14,13,15, // right
	16,17,18, 18,17,19, // top
	20,21,22, 22,21,23, // bottom
];


const CUBE_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D u_texture;

void main() {
	o_fragColor = texture(u_texture, v_uv) * v_color;
}
"#;

const CUBE_VS: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec2 a_uv;
in vec4 a_color;

out vec4 v_color;
out vec2 v_uv;

uniform mat4 u_transform;

void main()
{
	v_color = a_color + vec4(0.5, 0.5, 0.5, 0.0);
	v_uv = a_uv;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

//----------------------------------------------------------------
// Cube renderable

struct CubeMaterial {
	shader: shade::Shader,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for CubeMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_texture", &self.texture);
	}
}

struct CubeInstance {
	model: Transform3f,
}
impl shade::UniformVisitor for CubeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

struct CubeRenderable {
	mesh: shade::d3::VertexIndexedMesh,
	material: CubeMaterial,
	instance: CubeInstance,
}

impl CubeRenderable {
	fn create(g: &mut shade::Graphics) -> CubeRenderable {
		// Create the vertex and index buffers
		let vertices = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static);
		let vertices_len: u32 = VERTICES.len() as u32;
		let indices = g.index_buffer(None, &INDICES, VERTICES.len() as u8, shade::BufferUsage::Static);
		let indices_len = INDICES.len() as u32;

		let mesh = shade::d3::VertexIndexedMesh {
			origin: Vec3f::ZERO,
			bounds: Bounds3f(Vec3f(X_MIN, Y_MIN, Z_MIN), Vec3f(X_MAX, Y_MAX, Z_MAX)),
			vertices,
			vertices_len,
			indices,
			indices_len,
		};

		// Load the texture
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/brick 24 - 256x256.png").unwrap();
			g.image(Some("brick 24"), &image)
		};

		// Create the shader
		let shader = g.shader_create(None, CUBE_VS, CUBE_FS);

		let material = CubeMaterial { shader, texture };

		let instance = CubeInstance { model: Transform3f::IDENTITY };

		CubeRenderable { mesh, material, instance }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::Camera, instance: &CubeInstance) {
		let transform = camera.view_proj * instance.model;

		// Draw the cube
		g.draw_indexed(&shade::DrawIndexedArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: None,
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader: self.material.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: self.mesh.indices,
			uniforms: &[
				camera,
				&self.material,
				&self.instance,
				&shade::UniformFn(|set| {
					set.value("u_transform", &transform);
				}),
			],
			index_start: 0,
			index_end: self.mesh.indices_len,
			instances: -1,
		});
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
	cube: CubeRenderable,
	model: Transform3f,
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

		let mut g = shade::gl::GlGraphics::new();
		let cube = CubeRenderable::create(&mut g);
		let model = Transform3::IDENTITY;

		Box::new(App { size, window, surface, context, g, cube, model })
	}

	fn draw(&mut self) {
		// Render the frame
		let viewport = Bounds2::c(0, 0, self.size.width as i32, self.size.height as i32);
		self.g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(self.g, color: Vec4(0.2, 0.5, 0.2, 1.0), depth: 1.0);

		// Rotate the cube
		self.model = self.model * Transform3::rotate(Vec3(0.8, 0.6, 0.1), Angle::deg(1.0));

		// Camera setup
		let camera = {
			let aspect_ratio = self.size.width as f32 / self.size.height as f32;
			let position = Vec3(0.0, 0.0, 4.0);
			let target = Vec3::ZERO;
			let view = Transform3f::look_at(position, target, Vec3::Y, Hand::RH);
			let (near, far) = (0.1, 100.0);
			let fov_y = Angle::deg(45.0);
			let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (Hand::RH, Clip::NO));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip: Clip::NO }
		};

		// Draw the cube
		self.cube.draw(&mut self.g, &camera, &CubeInstance { model: self.model });

		// Finish the frame
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
