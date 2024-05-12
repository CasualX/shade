use std::{thread::sleep, time::Duration};

//----------------------------------------------------------------
// The bunny's geometry

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MyVertex3 {
	position: cvmath::Vec3<f32>,
	normal: cvmath::Vec3<f32>,
}

unsafe impl shade::TVertex for MyVertex3 {
	const VERTEX_LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: std::mem::size_of::<MyVertex3>() as u16,
		alignment: std::mem::align_of::<MyVertex3>() as u16,
		attributes: &[
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 3,
				offset: dataview::offset_of!(MyVertex3.position) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 3,
				offset: dataview::offset_of!(MyVertex3.normal) as u16,
			},
		],
	};
}

//----------------------------------------------------------------
// Shader and uniforms

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec3 Normal;

void main() {
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));
	float diff = max(dot(Normal, lightDir), 0.0);
	FragColor = vec4(1.0, 1.0, 1.0, 1.0) * (0.2 + diff * 0.5);
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

out vec3 Normal;

uniform mat4x4 transform;

void main()
{
	Normal = aNormal;
	gl_Position = transform * vec4(aPos, 1.0);
}
"#;

#[derive(Copy, Clone, dataview::Pod)]
#[repr(C)]
struct MyUniform3 {
	transform: cvmath::Mat4<f32>,
}

impl Default for MyUniform3 {
	fn default() -> Self {
		MyUniform3 {
			transform: cvmath::Mat4::IDENTITY,
		}
	}
}

unsafe impl shade::TUniform for MyUniform3 {
	const UNIFORM_LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: std::mem::size_of::<MyUniform3>() as u16,
		alignment: std::mem::align_of::<MyUniform3>() as u16,
		attributes: &[
			shade::UniformAttribute {
				name: "transform",
				ty: shade::UniformType::Mat4x4 { order: shade::UniformMatOrder::RowMajor },
				offset: dataview::offset_of!(MyUniform3.transform) as u16,
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

	let mut bunny_file = std::fs::File::open("examples/models/Bunny-LowPoly.stl").unwrap();
	let bunny_stl = stl::read_stl(&mut bunny_file).unwrap();

	let mut mins = cvmath::Vec3::dup(f32::INFINITY);
	let mut maxs = cvmath::Vec3::dup(f32::NEG_INFINITY);
	let (vb, vb_len) = {
		let mut vertices = Vec::new();
		for triangle in bunny_stl.triangles.iter() {
			vertices.push(MyVertex3 {
				position: triangle.v1.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(MyVertex3 {
				position: triangle.v2.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(MyVertex3 {
				position: triangle.v3.into(),
				normal: triangle.normal.into(),
			});
			mins = mins.min(triangle.v1.into()).min(triangle.v2.into()).min(triangle.v3.into());
			maxs = maxs.max(triangle.v1.into()).max(triangle.v2.into()).max(triangle.v3.into());
		}

		// Smooth the normals
		let mut map = std::collections::HashMap::new();
		for v in vertices.iter() {
			map.entry(v.position.map(f32::to_bits)).or_insert(Vec::new()).push(v.normal);
		}
		for v in &mut vertices {
			let normals = map.get(&v.position.map(f32::to_bits)).unwrap();
			let mut normal = cvmath::Vec3::ZERO;
			for n in normals.iter() {
				normal += *n;
			}
			v.normal = normal.normalize();
		};

		// Create the vertex and index buffers
		let vb = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();
		(vb, dbg!(vertices.len()) as u32)
	};

	println!("Bunny bounding box: {:?}", cvmath::Bounds(mins, maxs));

	// Create the uniform buffer
	// The uniform buffer is updated every frame
	let ub = g.uniform_buffer(None, &[
		MyUniform3::default(),
	]).unwrap();

	// Create the shader
	let shader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(shader, VERTEX_SHADER, FRAGMENT_SHADER) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}

	// Model matrix to rotate the bunny
	let mut model = cvmath::Mat4::rotate(cvmath::Deg(-90.0), cvmath::Vec3::X) * cvmath::Mat4::translate(-(mins + maxs) * 0.5);

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
			color: Some(cvmath::Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Rotate the bunny
		model = model * cvmath::Mat4::rotate(cvmath::Deg(1.0), cvmath::Vec3::Z);

		// Update the transformation matrices
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(45.0), size.width as f32, size.height as f32, 0.1, 1000.0, (cvmath::RH, cvmath::NO));
		let view = cvmath::Mat4::look_at(cvmath::Vec3(0.0, 50.0, -200.0), cvmath::Vec3(0.0, 0.0, 0.0), cvmath::Vec3(0.0, 1.0, 0.0), cvmath::RH);
		let transform = projection * view * model;

		// Update the uniform buffer with the new transformation matrix
		g.uniform_buffer_set_data(ub, &[
			MyUniform3 { transform },
		]).unwrap();

		// Draw the bunny
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Rect::c(0, 0, size.width as i32, size.height as i32),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: None,
			prim_type: shade::PrimType::Triangles,
			shader,
			vertices: vb,
			uniforms: ub,
			vertex_start: 0,
			vertex_end: vb_len,
			uniform_index: 0,
			instances: -1,
		}).unwrap();

		// Finish the frame
		g.end().unwrap();

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		sleep(Duration::from_millis(16));
	}
}
