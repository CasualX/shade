use std::{mem, thread, time};
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

layout(location = 0) out vec4 o_fragColor;

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

#[derive(Clone, Debug)]
struct CubeUniforms {
	transform: Mat4f,
	texture: shade::Texture2D,
}

impl shade::UniformVisitor for CubeUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
	}
}

//----------------------------------------------------------------
// Model and instance

struct CubeInstance {
	model: Transform3f,
}

struct CubeModel {
	shader: shade::Shader,
	texture: shade::Texture2D,
	vertices: shade::VertexBuffer,
	indices: shade::IndexBuffer,
	indices_len: u32,
}

impl CubeModel {
	fn create(g: &mut shade::Graphics) -> CubeModel {
		// Load the texture
		let texture = shade::image::png::load_file(g, Some("brick 24"), "examples/textures/brick 24 - 256x256.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		// Create the vertex and index buffers
		let vertices = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static);
		let indices = g.index_buffer(None, &INDICES, VERTICES.len() as u8, shade::BufferUsage::Static);
		let indices_len = INDICES.len() as u32;

		// Create the shader
		let shader = g.shader_create(None, CUBE_VS, CUBE_FS);

		CubeModel { shader, texture, vertices, indices, indices_len }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &CubeInstance) {
		let transform = camera.view_proj * instance.model;
		let uniforms = CubeUniforms { transform, texture: self.texture };

		// Draw the cube
		g.draw_indexed(&shade::DrawIndexedArgs {
			surface: camera.surface,
			viewport: camera.viewport,
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: None,
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: self.indices,
			uniforms: &[&uniforms],
			index_start: 0,
			index_end: self.indices_len,
			instances: -1,
		});
	}
}

//----------------------------------------------------------------

fn main() {
	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let mut event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_inner_size(size);

	let window_context = glutin::ContextBuilder::new()
		.with_multisampling(4)
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let ref mut g = shade::gl::GlGraphics::new();

	// Create the cube model
	let cube = CubeModel::create(g);

	// Model matrix to rotate the cube
	let mut model = Transform3::scale(1.0);

	// Main loop
	let mut quit = false;
	while !quit {
		// Handle events
		use winit::platform::run_return::EventLoopExtRunReturn as _;
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = winit::event_loop::ControlFlow::Wait;

			// // Print only Window events to reduce noise
			// if let winit::event::Event::WindowEvent { event, .. } = &event {
			// 	println!("{:?}", event);
			// }

			match event {
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
					quit = true;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
					size = new_size;
					context.resize(new_size);
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		// Render the frame
		g.begin();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.5, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		// Rotate the cube
		model = model * Transform3::rotate(Vec3(0.8, 0.6, 0.1), Deg(1.0));

		// Camera setup
		let camera = {
			let surface = shade::Surface::BACK_BUFFER;
			let viewport = Bounds2::c(0, 0, size.width as i32, size.height as i32);
			let aspect_ratio = size.width as f32 / size.height as f32;
			let position = Vec3(0.0, 0.0, 4.0);
			let target = Vec3::ZERO;
			let view = Transform3f::look_at(position, target, Vec3::Y, Hand::RH);
			let (near, far) = (0.1, 100.0);
			let fov_y = Deg(45.0);
			let projection = Mat4::perspective_fov(fov_y, size.width as f32, size.height as f32, near, far, (Hand::RH, Clip::NO));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { surface, viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip: Clip::NO }
		};

		// Draw the cube
		cube.draw(g, &camera, &CubeInstance { model });

		// Finish the frame
		g.end();

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
