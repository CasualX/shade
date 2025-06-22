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
			shade::VertexAttribute::with::<cvmath::Vec3f>("a_pos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<cvmath::Vec3f>("a_normal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(Vertex.color)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec3 v_normal;
in vec4 v_color;
in vec3 v_fragpos;

uniform vec3 u_lightpos;

void main() {
	vec3 lightDir = normalize(u_lightpos - v_fragpos);
	float diff = max(dot(v_normal, lightDir), 0.0);
	FragColor = vec4(1.0, 1.0, 1.0, 1.0) * (0.2 + diff * 0.5) * v_color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
in vec3 a_pos;
in vec3 a_normal;
in vec4 a_color;

out vec3 v_normal;
out vec4 v_color;
out vec3 v_fragpos;

uniform mat4x4 u_transform;
uniform mat4x4 u_model;

void main()
{
	v_normal = a_normal;
	v_color = a_color;
	v_fragpos = (u_model * vec4(a_pos, 1.0)).xyz;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

//----------------------------------------------------------------

struct ColorTreeInstance {
	model: cvmath::Mat4f,
	light_pos: cvmath::Vec3f,
}

impl shade::UniformVisitor for ColorTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		set.value("u_lightpos", &self.light_pos);
	}
}

struct ColorTreeModel {
	shader: shade::Shader,
	vertices: shade::VertexBuffer,
	vertices_len: u32,
}

impl shade::UniformVisitor for ColorTreeModel {
	fn visit(&self, _set: &mut dyn shade::UniformSetter) {
		// No additional uniforms needed for this model
	}
}

impl ColorTreeModel {
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::camera::CameraSetup, instance: &ColorTreeInstance) {
		let transform = camera.view_proj * instance.model;
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
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
			uniforms: &[camera, self, instance, &("u_transform", &transform)],
			vertex_start: 0,
			vertex_end: self.vertices_len,
			instances: -1,
		}).unwrap();
	}
}

//----------------------------------------------------------------

struct Scene {
	screen_size: cvmath::Vec2<i32>,
	camera: shade::camera::ArcballCamera,
	color_tree: ColorTreeModel,
	axes: shade::d3::axes::AxesModel,
}

impl Scene {
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
		let viewport = cvmath::Bounds2::vec(self.screen_size);
		let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
		let position = self.camera.position();
		let hand = cvmath::Hand::RH;
		let clip = cvmath::Clip::NO;
		let near = 0.1;
		let far = 10000.0;
		let view = self.camera.view_matrix(hand);
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(90.0), self.screen_size.x as f32, self.screen_size.y as f32, near, far, (hand, clip));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		let camera = shade::camera::CameraSetup { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip };

		let light_pos = cvmath::Vec3f::new(10000.0, 10000.0, 10000.0);

		// Draw the model
		self.color_tree.draw(g, &camera, &ColorTreeInstance {
			model: cvmath::Mat4f::IDENTITY,
			light_pos,
		});

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: cvmath::Mat4::scale(position.len() * 0.2),
		});

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

	let color_tree = ColorTreeModel {
		shader,
		vertices: vb,
		vertices_len: vb_len,
	};

	let axes = {
		let shader = g.shader_create(None, include_str!("../src/gl/shaders/gizmo.axes.vs.glsl"), include_str!("../src/gl/shaders/gizmo.axes.fs.glsl")).unwrap();
		shade::d3::axes::AxesModel::create(&mut g, shader)
	};

	let mut scene = Scene {
		screen_size: cvmath::Vec2::new(size.width as i32, size.height as i32),
		camera: shade::camera::ArcballCamera::new(camera_position, target, cvmath::Vec3::Z),
		color_tree,
		axes,
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
					scene.screen_size.x = new_size.width as i32;
					scene.screen_size.y = new_size.height as i32;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					let dx = position.x as f32 - cursor_position.x as f32;
					let dy = position.y as f32 - cursor_position.y as f32;
					if left_click {
						auto_rotate = false;
						scene.camera.rotate(dx, dy);
					}
					if right_click {
						auto_rotate = false;
						scene.camera.pan(-dx, dy);
					}
					if middle_click {
						scene.camera.zoom(dy * 0.01);
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
			scene.camera.rotate(1.0, 0.0);
		}

		scene.draw(&mut g);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
