use std::{fs, mem, slice, thread, time};

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: cvmath::Vec3f,
	normal: cvmath::Vec3f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<cvmath::Vec3f>("aPos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<cvmath::Vec3f>("aNormal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("aColor", dataview::offset_of!(Vertex.color)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec4 Color;

void main() {
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));
	float diff = max(dot(Normal, lightDir), 0.0);
	FragColor = vec4(1.0, 1.0, 1.0, 1.0) * (0.2 + diff * 0.5) * Color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec4 aColor;

out vec3 Normal;
out vec4 Color;

uniform mat4x4 transform;

void main()
{
	Normal = aNormal;
	Color = aColor;
	gl_Position = transform * vec4(aPos, 1.0);
}
"#;

#[derive(Copy, Clone)]
#[repr(C)]
struct Uniform {
	transform: cvmath::Mat4f,
}

impl Default for Uniform {
	fn default() -> Self {
		Uniform {
			transform: cvmath::Mat4::IDENTITY,
		}
	}
}

unsafe impl shade::TUniform for Uniform {
	const LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: mem::size_of::<Uniform>() as u16,
		alignment: mem::align_of::<Uniform>() as u16,
		fields: &[
			shade::UniformField {
				name: "transform",
				ty: shade::UniformType::Mat4x4 { layout: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.transform) as u16,
				len: 1,
			},
		],
	};
}


//----------------------------------------------------------------

struct State {
	screen_size: cvmath::Vec2<i32>,
	camera: shade::camera::ArcballCamera,
	model_shader: shade::Shader,
	model_vertices: shade::VertexBuffer,
	model_vertices_len: u32,
}

impl State {
	fn draw(&mut self, g: &mut shade::Graphics) {
		// Render the frame
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Update the transformation matrices
		let model = cvmath::Mat4::IDENTITY;
		let view = self.camera.view_matrix(cvmath::RH);
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(90.0), self.screen_size.x as f32, self.screen_size.y as f32, 0.1, 10000.0, (cvmath::RH, cvmath::NO));
		let transform = projection * view * model;

		// Update the uniform buffer with the new transformation matrix
		let uniforms = Uniform { transform };

		// Draw the model
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::vec(self.screen_size),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: None,
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader: self.model_shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.model_vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[shade::UniformRef::from(&uniforms)],
			vertex_start: 0,
			vertex_end: self.model_vertices_len,
			instances: -1,
		}).unwrap();

		// Finish the frame
		g.end().unwrap();
	}
}

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

	let (vb, vb_len, mut mins, mut maxs); {
		let vertices = fs::read("examples/colortree/vertices.bin").unwrap();
		let vertices = unsafe { slice::from_raw_parts(vertices.as_ptr() as *const Vertex, vertices.len() / mem::size_of::<Vertex>()) };
		vb = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();
		vb_len = vertices.len() as u32;
		mins = cvmath::Vec3::dup(f32::INFINITY);
		maxs = cvmath::Vec3::dup(f32::NEG_INFINITY);
		for v in vertices {
			mins = mins.min(v.position);
			maxs = maxs.max(v.position);
			// println!("Vertex {}: {:?}", i, v);
		}
	}

	let extent = maxs - mins;
	let target = (maxs + mins) * 0.5;
	let camera_position = target + cvmath::Vec3::<f32>::X * f32::max(extent.x, extent.y) * 1.0;

	// Create the shader
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

	let mut state = State {
		screen_size: cvmath::Vec2::new(size.width as i32, size.height as i32),
		camera: shade::camera::ArcballCamera::new(camera_position, target, cvmath::Vec3::Z),
		model_shader: shader,
		model_vertices: vb,
		model_vertices_len: vb_len,
	};

	let mut left_click = false;
	let mut right_click = false;
	let mut middle_click = false;
	let mut auto_rotate = true;
	let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

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
					state.screen_size.x = new_size.width as i32;
					state.screen_size.y = new_size.height as i32;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					let dx = position.x as f32 - cursor_position.x as f32;
					let dy = position.y as f32 - cursor_position.y as f32;
					if left_click {
						auto_rotate = false;
						state.camera.rotate(-dx, dy);
					}
					if right_click {
						auto_rotate = false;
						state.camera.pan(dx, dy);
					}
					if middle_click {
						state.camera.zoom(dy * 0.01);
					}
					cursor_position = position;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. }, .. } => {
					left_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Right, .. }, .. } => {
					right_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Middle, .. }, .. } => {
					middle_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		if auto_rotate {
			state.camera.rotate(1.0, 0.0);
		}

		state.draw(&mut g);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
