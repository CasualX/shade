use std::collections::HashMap;
use std::{fs, mem, thread, time};

use shade::cvmath::*;

//----------------------------------------------------------------
// Geometry, uniforms and shader

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct BunnyVertex {
	position: Vec3f,
	normal: Vec3f,
}

unsafe impl shade::TVertex for BunnyVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<BunnyVertex>() as u16,
		alignment: mem::align_of::<BunnyVertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(BunnyVertex.position)),
			shade::VertexAttribute::with::<Vec3f>("a_normal", dataview::offset_of!(BunnyVertex.normal)),
		],
	};
}

#[derive(Clone, Debug)]
struct BunnyUniforms {
	transform: Mat4f,
	light_dir: Vec3f,
}

impl shade::UniformVisitor for BunnyUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_lightDir", &self.light_dir);
	}
}

const BUNNY_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_normal;

uniform vec3 u_lightDir;

void main() {
	float diffuseLight = max(dot(v_normal, u_lightDir), 0.0);
	o_fragColor = vec4(1.0, 1.0, 1.0, 1.0) * (0.2 + diffuseLight * 0.5);
}
"#;

const BUNNY_VS: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;

out vec3 v_normal;

uniform mat4 u_transform;

void main()
{
	v_normal = a_normal;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

//----------------------------------------------------------------
// Model and instance

struct BunnyInstance {
	model: Transform3f,
	light_dir: Vec3f,
}

struct BunnyModel {
	shader: shade::Shader,
	vertices: shade::VertexBuffer,
	vertices_len: u32,
	bounds: Bounds3<f32>,
}

impl BunnyModel {
	fn create(g: &mut shade::Graphics) -> BunnyModel {
		let mut bunny_file = fs::File::open("examples/models/Bunny-LowPoly.stl").unwrap();
		let bunny_stl = stl::read_stl(&mut bunny_file).unwrap();

		let mut mins = Vec3::dup(f32::INFINITY);
		let mut maxs = Vec3::dup(f32::NEG_INFINITY);
		let mut vertices = Vec::new();
		for triangle in bunny_stl.triangles.iter() {
			vertices.push(BunnyVertex {
				position: triangle.v1.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(BunnyVertex {
				position: triangle.v2.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(BunnyVertex {
				position: triangle.v3.into(),
				normal: triangle.normal.into(),
			});
			mins = mins.min(triangle.v1.into()).min(triangle.v2.into()).min(triangle.v3.into());
			maxs = maxs.max(triangle.v1.into()).max(triangle.v2.into()).max(triangle.v3.into());
		}
		let bounds = Bounds(mins, maxs);

		// Smooth the normals
		let mut map = HashMap::new();
		for v in vertices.iter() {
			map.entry(v.position.map(f32::to_bits)).or_insert(Vec::new()).push(v.normal);
		}
		for v in &mut vertices {
			let normals = map.get(&v.position.map(f32::to_bits)).unwrap();
			let mut normal = Vec3::ZERO;
			for n in normals.iter() {
				normal += *n;
			}
			v.normal = normal.normalize();
		};

		// Create the vertex and index buffers
		let vertices_len = vertices.len() as u32;
		let vertices = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static);

		println!("Bunny # vertices: {vertices_len}");
		println!("Bunny bounds: {bounds:#?}");

		// Create the shader
		let shader = g.shader_create(None, BUNNY_VS, BUNNY_FS);

		BunnyModel { shader, vertices, vertices_len, bounds }
	}

	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &BunnyInstance) {
		let transform = camera.view_proj * instance.model;
		let light_dir = instance.light_dir;
		let uniforms = BunnyUniforms { transform, light_dir };

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
			uniforms: &[&uniforms],
			vertex_start: 0,
			vertex_end: self.vertices_len,
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

	// Create the bunny model
	let bunny = BunnyModel::create(g);

	// Create the axes gizmo
	let axes = {
		let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
		shade::d3::axes::AxesModel::create(g, shader)
	};

	// Bunny model transform
	let mut bunny_rotation = Transform3::IDENTITY;

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
			color: Some(Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		// Camera setup
		let camera = {
			let surface = shade::Surface::BACK_BUFFER;
			let viewport = Bounds2::c(0, 0, size.width as i32, size.height as i32);
			let aspect_ratio = size.width as f32 / size.height as f32;
			let position = Vec3(100.0, 100.0, bunny.bounds.height());
			let target = Vec3::ZERO;
			let view = Transform3f::look_at(position, target, Vec3::Z, Hand::RH);
			let (near, far) = (0.1,1000.0);
			let fov_y = Deg(45.0);
			let projection = Mat4::perspective_fov(fov_y, size.width as f32, size.height as f32, near, far, (Hand::RH, Clip::NO));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { surface, viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip: Clip::NO }
		};

		// Rotate the bunny
		bunny_rotation *= Transform3::rotate(Vec3::Z, Deg(1.0));

		// Draw the bunny
		let model = bunny_rotation * Transform3::translate(-bunny.bounds.center());
		let light_dir = Vec3::new(1.0, -1.0, 1.0).normalize();
		bunny.draw(g, &camera, &BunnyInstance { model, light_dir });

		// Draw the axes gizmo
		axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(camera.position.len() * 0.32),
			depth_test: Some(shade::DepthTest::Less),
		});

		// Finish the frame
		g.end();

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
