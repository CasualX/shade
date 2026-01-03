use std::collections::HashMap;
use std::{fs, mem, time};
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

const SIZE: i32 = 256;

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


const POST_PROCESS_FS: &str = r#"\
#version 330 core
out vec4 o_fragColor;
in vec2 v_uv;
uniform sampler2D u_texture;
void main() {
	o_fragColor = texture(u_texture, v_uv);
}
"#;

struct PostProcessUniforms {
	texture: shade::Texture2D,
}

impl shade::UniformVisitor for PostProcessUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
	}
}

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
		let bounds = Bounds3(mins, maxs);

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
			v.normal = normal.norm();
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
// Application state

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	bunny: BunnyModel,
	axes: shade::d3::axes::AxesModel,
	bunny_rotation: Transform3f,
	start_time: time::Instant,
	texture: shade::Texture2D,
	texture_depth: shade::Texture2D,
	post_process_shader: shade::Shader,
	post_process_quad: shade::d2::PostProcessQuad,
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
		let bunny = BunnyModel::create(&mut g);
		let axes = {
			let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
			shade::d3::axes::AxesModel::create(&mut g, shader)
		};
		let bunny_rotation = Transform3::IDENTITY;
		let start_time = time::Instant::now();
		let texture = g.texture2d_create(None, &shade::Texture2DInfo {
			format: shade::TextureFormat::RGBA8,
			levels: 1,
			width: SIZE,
			height: SIZE,
			props: shade::TextureProps {
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			},
		});
		let texture_depth = g.texture2d_create(None, &shade::Texture2DInfo {
			format: shade::TextureFormat::Depth24,
			levels: 1,
			width: SIZE,
			height: SIZE,
			props: shade::TextureProps::default(),
		});

		let post_process_shader = g.shader_create(None, shade::gl::shaders::POST_PROCESS_VS, POST_PROCESS_FS);
		let post_process_quad = shade::d2::PostProcessQuad::create(&mut g);

		Box::new(App { size, window, surface, context, g, bunny, axes, bunny_rotation, start_time, texture, texture_depth, post_process_shader, post_process_quad })
	}

	fn draw(&mut self) {
		let viewport = Bounds2::c(0, 0, SIZE, SIZE);
		self.g.begin(&shade::RenderPassArgs::Immediate {
			color: &[self.texture],
			depth: self.texture_depth,
			viewport,
		});

		// Clear the screen
		self.g.clear(&shade::ClearArgs {
			color: Some(Vec4(0.0, 0.0, 0.0, 0.0)),
			depth: Some(1.0),
			..Default::default()
		});

		// Camera setup
		let camera = {
			let aspect_ratio = self.size.width as f32 / self.size.height as f32;
			let position = Vec3(-50.0, -200.0, self.bunny.bounds.height() * 0.5);
			let target = Vec3::ZERO;
			let view = Transform3f::look_at(position, target, Vec3::Z, Hand::RH);

			let blend = (self.start_time.elapsed().as_secs_f32() * 0.5).fract() * 2.0;
			let blend = if blend > 1.0 { 2.0 - blend } else { blend };
			let focus_depth = position.distance(target);
			let fov_y = Angle::deg(45.0);
			let near = 0.1;
			let far = 1000.0;
			let projection = Mat4f::blend_ortho_perspective(blend, focus_depth, fov_y, aspect_ratio, near, far, (Hand::RH, Clip::NO));

			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip: Clip::NO }
		};

		// Rotate the bunny
		self.bunny_rotation *= Transform3::rotate(Vec3::Z, Angle::deg(1.0));

		// Draw the bunny
		let model = self.bunny_rotation * Transform3::translate(-self.bunny.bounds.center());
		let light_dir = Vec3::new(1.0, -1.0, 1.0).norm();
		self.bunny.draw(&mut self.g, &camera, &BunnyInstance { model, light_dir });

		// Draw the axes gizmo
		self.axes.draw(&mut self.g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(Vec3::dup(camera.position.len() * 0.32)),
			depth_test: Some(shade::DepthTest::Less),
		});

		self.g.end();

		// Render the frame
		let viewport = Bounds2::c(0, 0, self.size.width as i32, self.size.height as i32);
		self.g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		// Clear the screen
		self.g.clear(&shade::ClearArgs {
			color: Some(Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		let uniforms = PostProcessUniforms { texture: self.texture };
		self.post_process_quad.draw(&mut self.g, self.post_process_shader, shade::BlendMode::Additive, &[&uniforms]);

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
