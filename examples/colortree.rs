use std::{fs, mem, slice, thread, time};
use shade::cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec3f,
	normal: Vec3f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<Vec3f>("a_normal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(Vertex.color)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"\
#version 330 core

out vec4 FragColor;

in vec3 v_normal;
in vec4 v_color;
in vec3 v_fragPos;

uniform vec3 u_lightPos;

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diff = max(dot(v_normal, lightDir), 0.0);
	FragColor = vec4(1.0, 1.0, 1.0, 1.0) * (0.2 + diff * 0.5) * v_color;
}
"#;

const VERTEX_SHADER: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;
in vec4 a_color;

out vec3 v_normal;
out vec4 v_color;
out vec3 v_fragPos;

uniform mat4x3 u_model;

uniform mat4 u_transform;

void main()
{
	v_normal = a_normal;
	v_color = a_color;
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

//----------------------------------------------------------------

struct ColorTreeInstance {
	model: Transform3f,
	light_pos: Vec3f,
}

impl shade::UniformVisitor for ColorTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		set.value("u_lightPos", &self.light_pos);
	}
}

struct ColorTreeModel {
	shader: shade::Shader,
	vertices: shade::VertexBuffer,
	vertices_len: u32,
	bounds: Bounds3<f32>,
}

impl shade::UniformVisitor for ColorTreeModel {
	fn visit(&self, _set: &mut dyn shade::UniformSetter) {
		// No additional uniforms needed for this model
	}
}

impl ColorTreeModel {
	fn create(g: &mut shade::Graphics) -> ColorTreeModel {
		let vertices = fs::read("examples/colortree/vertices.bin").unwrap();
		let vertices = unsafe { slice::from_raw_parts(vertices.as_ptr() as *const Vertex, vertices.len() / mem::size_of::<Vertex>()) };
		let mut mins = Vec3::dup(f32::INFINITY);
		let mut maxs = Vec3::dup(f32::NEG_INFINITY);
		for v in vertices {
			mins = mins.min(v.position);
			maxs = maxs.max(v.position);
			// println!("Vertex {}: {:?}", i, v);
		}
		let bounds = Bounds3(mins, maxs);

		let vertices_len = vertices.len() as u32;
		let vertices = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();

		// Create the shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

		ColorTreeModel { shader, vertices, vertices_len, bounds }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &ColorTreeInstance) {
		let transform = camera.view_proj * instance.model;
		g.draw(&shade::DrawArgs {
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
			uniforms: &[camera, self, instance, &("u_transform", &transform)],
			vertex_start: 0,
			vertex_end: self.vertices_len,
			instances: -1,
		}).unwrap();
	}
}

//----------------------------------------------------------------

struct Scene {
	screen_size: Vec2i,
	camera: shade::d3::ArcballCamera,
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
			color: Some(Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Camera setup
		let camera = {
			let surface = shade::Surface::BACK_BUFFER;
			let viewport = Bounds2::vec(self.screen_size);
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (10.0, 10000.0);
			let fov_y = Deg(90.0);
			let projection = Mat4::perspective_fov(fov_y, self.screen_size.x as f32, self.screen_size.y as f32, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { surface, viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		let light_pos = Vec3f::new(10000.0, 10000.0, 10000.0);

		// Draw the model
		self.color_tree.draw(g, &camera, &ColorTreeInstance {
			model: Transform3f::IDENTITY,
			light_pos,
		});

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(camera.position.len() * 0.2),
			depth_test: None,
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
		.with_multisampling(4)
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let ref mut g = shade::gl::GlGraphics::new();

	// Create the scene
	let mut scene = {
		let color_tree = ColorTreeModel::create(g);

		let axes = {
			let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS).unwrap();
			shade::d3::axes::AxesModel::create(g, shader)
		};

		let camera = {
			let pivot = color_tree.bounds.center().set_x(0.0).set_y(0.0);
			let position = pivot + Vec3::<f32>::X * color_tree.bounds.size().xy().vmax() * 1.0;

			shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
		};

		let screen_size = Vec2::new(size.width as i32, size.height as i32);

		Scene { screen_size, camera, color_tree, axes }
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
						scene.camera.rotate(-dx, -dy);
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
			scene.camera.rotate(-1.0, 0.0);
		}

		scene.draw(g);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
