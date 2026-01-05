use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

//----------------------------------------------------------------
// Uniforms and shaders

const SPHERE_FS: &str = r#"\
#version 330 core

in vec3 vWorldPos;
out vec4 FragColor;

uniform vec3 u_camera_pos;
uniform vec3 u_sphere_center;
uniform float u_sphere_radius;
uniform sampler2D u_texture;

const float PI = 3.141592653589793;

void main()
{
	// Ray from camera through fragment
	vec3 rayDir = normalize(vWorldPos - u_camera_pos);
	vec3 rayOrigin = u_camera_pos;

	// Sphere centered at `u_sphere_center` (world space)
	vec3 oc = rayOrigin - u_sphere_center;

	float a = dot(rayDir, rayDir);
	float b = 2.0 * dot(oc, rayDir);
	float c = dot(oc, oc) - u_sphere_radius * u_sphere_radius;

	float discriminant = b*b - 4.0*a*c;
	if (discriminant < 0.0) {
	// Purple debug color
		FragColor = vec4(0.5, 0.0, 0.5, 1.0);
		return;
		discard;
	}
	// Nearest positive intersection (handles camera-inside-sphere too)
	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) discard;

	vec3 hitPos = rayOrigin + t * rayDir;
	vec3 n = normalize(hitPos - u_sphere_center);

	// Spherical UVs (equirectangular)
	// World is Z-up in this demo (see ArcballCamera::new(..., up = Z)).
	// Longitude around +Z axis, latitude from equator toward +Z.
	float u = 0.5 + atan(n.y, n.x) / (2.0 * PI);
	float v = 0.5 + asin(n.z) / PI;

	// PNG rows decode top-to-bottom; OpenGL UV (0,0) samples the first row.
	// Flip V so the image appears upright.
	v = 1.0 - v;

	// Keep within [0,1) for wrapping samplers.
	u = fract(u);

	vec3 color = texture(u_texture, vec2(u, v)).rgb;
	FragColor = vec4(color, 1.0);
}
"#;

const SPHERE_VS: &str = r#"\
#version 330 core

layout (location = 0) in vec3 a_pos;

uniform mat4x3 u_view;
uniform mat4 u_projection;

uniform vec3 u_sphere_center;
uniform float u_sphere_radius;

out vec3 vWorldPos;

void main()
{
	// The mesh is a unit icosahedron in [-1, 1]^3. Scale it to radius (R) and translate.
	vec3 world = u_sphere_center + a_pos * (1.27 * u_sphere_radius);
	vec4 worldPos = vec4(world, 1.0);
	vWorldPos = worldPos.xyz;
	gl_Position = u_projection * mat4(u_view) * worldPos;
}
"#;

//----------------------------------------------------------------
// Model and instance

struct SphereInstance {
	sphere_center: Vec3f,
	sphere_radius: f32,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for SphereInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_sphere_center", &self.sphere_center);
		set.value("u_sphere_radius", &self.sphere_radius);
		set.value("u_texture", &self.texture);
	}
}

type MeshModel = shade::d3::icosahedron::IcosahedronFlatModel;

struct SphereModel {
	shader: shade::Shader,
	texture: shade::Texture2D,
	mesh: MeshModel,
}

impl SphereModel {
	fn create(g: &mut shade::Graphics) -> SphereModel {
		let texture = {
			let image = shade::image::DecodedImage::load_file_jpeg("examples/textures/2k_earth_daymap.jpg").unwrap().rgb().unwrap();
			g.image(None, &image)
		};

		let mesh = MeshModel::create(g);
		let shader = g.shader_create(None, SPHERE_VS, SPHERE_FS);

		SphereModel { shader, texture, mesh }
	}

	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &SphereInstance) {
		self.mesh.draw(g, self.shader, &[camera, instance]);
	}
}

//----------------------------------------------------------------
// Scene

struct Scene {
	screen_size: Vec2i,
	camera: shade::d3::ArcballCamera,
	demo: SphereModel,
	axes: shade::d3::axes::AxesModel,
}

impl Scene {
	fn draw(&mut self, g: &mut shade::Graphics) {
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		shade::clear!(g, color: Vec4(0.1, 0.1, 0.1, 1.0), depth: 1.0);

		// Camera setup
		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 100.0);
			let fov_y = Angle::deg(45.0);
			let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip }
		};

		self.demo.draw(g, &camera, &SphereInstance {
			sphere_center: Vec3f::ZERO,
			sphere_radius: 0.8,
			texture: self.demo.texture,
		});

		// Axes gizmo (fixed scale; no dynamic scaling)
		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::IDENTITY,
			depth_test: Some(shade::DepthTest::Less),
		});

		g.end();
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

		let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default().with_inner_size(size);

		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template, |configs| configs.max_by_key(|c| c.num_samples()).unwrap())
			.expect("Failed to build window and GL config");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window.window_handle().expect("Failed to get raw window handle").as_raw();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe { gl_display.create_context(&gl_config, &context_attributes) }
			.expect("Failed to create GL context");

		let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs) }
			.expect("Failed to create GL surface");

		let context = not_current.make_current(&surface).expect("Failed to make GL context current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		let mut g = shade::gl::GlGraphics::new();

		let scene = {
			let demo = SphereModel::create(&mut g);
			let axes = {
				let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
				shade::d3::axes::AxesModel::create(&mut g, shader)
			};
			let camera = shade::d3::ArcballCamera::new(Vec3(0.0, 3.2, 1.8), Vec3::ZERO, Vec3f::Z);
			let screen_size = Vec2::new(size.width as i32, size.height as i32);
			Scene { screen_size, camera, demo, axes }
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
							auto_rotate = false;
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
