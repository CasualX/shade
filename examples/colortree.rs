use std::mem;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
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

shade::include_bin!(VERTICES: [Vertex] = "colortree/vertices.bin");

impl ColorTreeModel {
	fn create(g: &mut shade::Graphics) -> ColorTreeModel {
		let vertices: &[Vertex] = VERTICES.as_slice();
		let mut mins = Vec3::dup(f32::INFINITY);
		let mut maxs = Vec3::dup(f32::NEG_INFINITY);
		for v in vertices {
			mins = mins.min(v.position);
			maxs = maxs.max(v.position);
			// println!("Vertex {}: {:?}", i, v);
		}
		let bounds = Bounds3(mins, maxs);

		let vertices_len = vertices.len() as u32;
		let vertices = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static);

		// Create the shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		ColorTreeModel { shader, vertices, vertices_len, bounds }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &ColorTreeInstance) {
		let transform = camera.view_proj * instance.model;
		g.draw(&shade::DrawArgs {
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
		});
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
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.5, 0.2, 0.2, 1.0), depth: 1.0);

		// Camera setup
		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (10.0, 10000.0);
			let fov_y = Angle::deg(90.0);
			let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		let light_pos = Vec3f::new(10000.0, 10000.0, 10000.0);

		// Draw the model
		self.color_tree.draw(g, &camera, &ColorTreeInstance {
			model: Transform3f::IDENTITY,
			light_pos,
		});

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(Vec3::dup(camera.position.len() * 0.2)),
			depth_test: None,
		});

		// Finish the frame
		g.end();
	}
}

//----------------------------------------------------------------

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	scene: Scene,
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

		// Create the scene
		let scene = {
			let color_tree = ColorTreeModel::create(&mut g);

			let axes = {
				let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
				shade::d3::axes::AxesModel::create(&mut g, shader)
			};

			let camera = {
				let pivot = color_tree.bounds.center().set_x(0.0).set_y(0.0);
				let position = pivot + Vec3::<f32>::X * color_tree.bounds.size().xy().vmax() * 1.0;

				shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
			};

			let screen_size = Vec2::new(size.width as i32, size.height as i32);

			Scene { screen_size, camera, color_tree, axes }
		};

		Box::new(App { size, window, surface, context, g, scene })
	}

	fn draw(&mut self) {
		self.scene.draw(&mut self.g);
	}
}

//----------------------------------------------------------------

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let mut app: Option<Box<App>> = None;

	let mut left_click = false;
	let mut right_click = false;
	let mut middle_click = false;
	let mut auto_rotate = true;
	let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, MouseButton, WindowEvent};

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
						app.scene.screen_size.x = new_size.width as i32;
						app.scene.screen_size.y = new_size.height as i32;
						app.surface.resize(&app.context, width, height);
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let dx = position.x as f32 - cursor_position.x as f32;
						let dy = position.y as f32 - cursor_position.y as f32;
						if left_click {
							auto_rotate = false;
							app.scene.camera.rotate(-dx, -dy);
						}
						if right_click {
							auto_rotate = false;
							app.scene.camera.pan(-dx, dy);
						}
						if middle_click {
							app.scene.camera.zoom(dy * 0.01);
						}
					}
					cursor_position = position;
				}
				WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
					left_click = matches!(state, ElementState::Pressed);
				}
				WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
					right_click = matches!(state, ElementState::Pressed);
				}
				WindowEvent::MouseInput { state, button: MouseButton::Middle, .. } => {
					middle_click = matches!(state, ElementState::Pressed);
				}
				WindowEvent::CloseRequested => event_loop.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						if auto_rotate {
							app.scene.camera.rotate(-1.0, 0.0);
						}
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
