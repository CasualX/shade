use std::{mem, thread, time};

//----------------------------------------------------------------
// The cube's geometry

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MyVertex3 {
	position: cvmath::Vec3f,
	tex_coord: cvmath::Vec2f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for MyVertex3 {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<MyVertex3>() as u16,
		alignment: mem::align_of::<MyVertex3>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<cvmath::Vec3f>("aPos", dataview::offset_of!(MyVertex3.position)),
			shade::VertexAttribute::with::<cvmath::Vec2f>("aTexCoord", dataview::offset_of!(MyVertex3.tex_coord)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("aColor", dataview::offset_of!(MyVertex3.color)),
		],
	};
}

const X_MIN: f32 = -1.0;
const X_MAX: f32 =  1.0;
const Y_MIN: f32 = -1.0;
const Y_MAX: f32 =  1.0;
const Z_MIN: f32 = -1.0;
const Z_MAX: f32 =  1.0;

static VERTICES: [MyVertex3; 24] = [
	// Front face
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([255,   0,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([192,   0,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([192,   0,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([128,   0,   0, 255]) },
	// Back face
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([  0, 255, 255, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([  0, 192, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([  0, 192, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([  0, 128, 128, 255]) },
	// Left face
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([  0, 255,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([  0, 192,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([  0, 192,   0, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([  0, 128,   0, 255]) },
	// Right face
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([255,   0, 255, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([192,   0, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([192,   0, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([128,   0, 128, 255]) },
	// Top face
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([  0,   0, 255, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MAX), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([  0,   0, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([  0,   0, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MAX, Z_MIN), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([  0,   0, 128, 255]) },
	// Bottom face
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(0.0, 0.0), color: shade::norm!([255, 255, 255, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MIN), tex_coord: cvmath::Vec2(1.0, 0.0), color: shade::norm!([192, 192, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MAX, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(0.0, 1.0), color: shade::norm!([192, 192, 192, 255]) },
	MyVertex3 { position: cvmath::Vec3(X_MIN, Y_MIN, Z_MAX), tex_coord: cvmath::Vec2(1.0, 1.0), color: shade::norm!([128, 128, 128, 255]) },
];

static INDICES: [u8; 36] = [
	 0, 1, 2,  2, 1, 3, // front
	 4, 5, 6,  6, 5, 7, // back
	 8, 9,10, 10, 9,11, // left
	12,13,14, 14,13,15, // right
	16,17,18, 18,17,19, // top
	20,21,22, 22,21,23, // bottom
];


//----------------------------------------------------------------
// Shader and uniforms

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec4 VertexColor;
in vec2 TexCoord;

uniform sampler2D tex;

void main() {
	FragColor = texture(tex, TexCoord) * VertexColor;
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
	VertexColor = aColor + vec4(0.5, 0.5, 0.5, 0.0);
	TexCoord = aTexCoord;
	gl_Position = transform * vec4(aPos, 1.0);
}
"#;

#[derive(Copy, Clone)]
#[repr(C)]
struct MyUniform3 {
	transform: cvmath::Mat4f,
	texture: shade::Texture2D,
}

impl Default for MyUniform3 {
	fn default() -> Self {
		MyUniform3 {
			transform: cvmath::Mat4::IDENTITY,
			texture: shade::Texture2D::INVALID,
		}
	}
}

unsafe impl shade::TUniform for MyUniform3 {
	const LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: mem::size_of::<MyUniform3>() as u16,
		alignment: mem::align_of::<MyUniform3>() as u16,
		fields: &[
			shade::UniformField {
				name: "transform",
				ty: shade::UniformType::Mat4x4 { layout: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(MyUniform3.transform) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "tex",
				ty: shade::UniformType::Sampler2D,
				offset: dataview::offset_of!(MyUniform3.texture) as u16,
				len: 1,
			},
		],
	};
}


//----------------------------------------------------------------

fn main() {
	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let mut event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_inner_size(size);

	let window_context = glutin::ContextBuilder::new()
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let mut g = shade::gl::GlGraphics::new();

	// Load the texture
	let texture = shade::image::png::load_file(&mut g, Some("brick 24"), "examples/textures/brick 24 - 256x256.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	// Create the vertex and index buffers
	let vb = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static).unwrap();
	let ib = g.index_buffer(None, &INDICES, shade::BufferUsage::Static).unwrap();

	// Create the shader
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

	// Model matrix to rotate the cube
	let mut model = cvmath::Mat4::scale(1.0);

	// Main loop
	let mut quit = false;
	while !quit {
		// Handle events
		use winit::platform::run_return::EventLoopExtRunReturn as _;
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = winit::event_loop::ControlFlow::Wait;

			if let winit::event::Event::WindowEvent { event, .. } = &event {
				// Print only Window events to reduce noise
				println!("{:?}", event);
			}

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
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.2, 0.5, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Rotate the cube
		model = model * cvmath::Mat4::rotate(cvmath::Deg(1.0), cvmath::Vec3(0.8, 0.6, 0.1));

		// Update the transformation matrices
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(45.0), size.width as f32, size.height as f32, 0.1, 100.0, (cvmath::RH, cvmath::NO));
		let view = cvmath::Mat4::look_at(cvmath::Vec3(0.0, 0.0, 4.0), cvmath::Vec3::ZERO, cvmath::Vec3(0.0, 1.0, 0.0), cvmath::RH);
		let transform = projection * view * model;

		// Update the uniform buffer with the new transformation matrix
		let uniforms = MyUniform3 { transform, texture };

		// Draw the cube
		g.draw_indexed(&shade::DrawIndexedArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::c(0, 0, size.width as i32, size.height as i32),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: None,
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: vb,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: ib,
			uniforms: &[shade::UniformRef::from(&uniforms)],
			vertex_start: 0,
			vertex_end: VERTICES.len() as u32,
			index_start: 0,
			index_end: INDICES.len() as u32,
			instances: -1,
		}).unwrap();

		// Finish the frame
		g.end().unwrap();

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
