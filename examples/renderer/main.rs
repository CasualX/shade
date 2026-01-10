use std::time;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

mod bunny;
mod colortree;
mod globe;
mod cube;
mod oldtree;
mod parallax;

struct Light {
	light_pos: Vec3f,
	light_view_proj: Mat4f,
	shadow_map: shade::Texture2D,
	shadow_texel_scale: f32,
}
impl shade::UniformVisitor for Light {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_lightPos", &self.light_pos);
		set.value("u_shadowMap", &self.shadow_map);
		set.value("u_shadowTexelScale", &self.shadow_texel_scale);
	}
}

struct Globals {
	time: f32,
}
impl shade::UniformVisitor for Globals {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
	}
}

//----------------------------------------------------------------

const SHADOW_MAP_SIZE: i32 = 4096;

fn animate_light_pos(t: f32) -> Vec3f {
	let center = Vec3f(0.0, 0.0, 80.0);
	let radius = 40.0;
	let period_s = 10.0;
	let angle = Angle::TURN / period_s * t;
	(center.xy() + angle.vec2() * radius).vec3(center.z)
}

struct Scene {
	screen_size: Vec2i,
	epoch: time::Instant,
	camera: shade::d3::ArcballCamera,
	shadow_map: shade::Texture2D,

	axes: shade::d3::axes::AxesModel,
	cube: cube::Renderable,
	bunny: bunny::Renderable,
	color_tree: colortree::Renderable,
	oldtree: oldtree::Renderable,
	parallax: parallax::Renderable,
	globe: globe::Renderable,
}
impl Scene {
	fn draw(&mut self, g: &mut shade::Graphics) {
		let time = self.epoch.elapsed().as_secs_f32();

		if self.shadow_map == shade::Texture2D::INVALID {
			self.shadow_map = g.texture2d_create(None, &shade::Texture2DInfo {
				width: SHADOW_MAP_SIZE,
				height: SHADOW_MAP_SIZE,
				format: shade::TextureFormat::Depth32F,
				props: shade::TextureProps {
					mip_levels: 1,
					usage: shade::TextureUsage!(SAMPLED | DEPTH_STENCIL_TARGET),
					filter_min: shade::TextureFilter::Linear,
					filter_mag: shade::TextureFilter::Linear,
					wrap_u: shade::TextureWrap::Edge,
					wrap_v: shade::TextureWrap::Edge,
					border_color: [0, 0, 0, 0],
				},
			});
		}

		let globals = Globals { time };
		let mut light = Light {
			light_pos: animate_light_pos(time),
			light_view_proj: Mat4::IDENTITY,
			shadow_map: self.shadow_map,
			shadow_texel_scale: 2.0,
		};

		// Render shadow map
		let viewport = Bounds2::vec(Vec2::dup(SHADOW_MAP_SIZE));
		g.begin(&shade::BeginArgs::Immediate {
			color: &[],
			depth: self.shadow_map,
			viewport,
		});

		// Light camera setup
		let light_camera = {
			let aspect_ratio = 1.0;
			let position = light.light_pos;
			let hand = Hand::RH;
			let view = Transform3f::look_at(position, Vec3(0.0, 0.0, 0.0), Vec3::Z, hand);
			let clip = Clip::NO;
			let (near, far) = (10.0, 10000.0);
			// let bounds = self.color_tree.mesh.bounds;
			// let projection = Transform3::ortho(bounds, (hand, clip)).mat4();
			let projection = Mat4::perspective(Angle::deg(90.0), aspect_ratio, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		light.light_view_proj = light_camera.view_proj;

		shade::clear!(g, depth: 1.0);
		self.cube.draw(g, &globals, &light_camera, &light, true);
		self.bunny.draw(g, &globals, &light_camera, &light, true);
		self.color_tree.draw(g, &globals, &light_camera, &light, true);
		self.oldtree.draw(g, &globals, &light_camera, &light, true);
		self.parallax.draw(g, &globals, &light_camera, &light, true);
		self.globe.draw(g, &globals, &light_camera, &light, true);
		g.end();

		// Render the frame
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.5, 0.2, 0.2, 1.0), depth: 1.0);

		// Camera setup
		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 1000.0);
			let fov_y = Angle::deg(90.0);
			let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		// Draw the model
		self.cube.draw(g, &globals, &camera, &light, false);
		self.bunny.draw(g, &globals, &camera, &light, false);
		self.color_tree.draw(g, &globals, &camera, &light, false);
		self.oldtree.draw(g, &globals, &camera, &light, false);
		self.parallax.draw(g, &globals, &camera, &light, false);
		self.globe.draw(g, &globals, &camera, &light, false);

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(Vec3::dup(camera.position.len() * 0.2)),
			depth_test: None,
		});

		// Finish the frame
		g.end();
	}
}

//----------------------------------------------------------------

/// OpenGL Window wrapper.
struct GlWindow {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
}

impl GlWindow {
	fn new(
		event_loop: &winit::event_loop::ActiveEventLoop,
		size: winit::dpi::PhysicalSize<u32>,
	) -> GlWindow {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_inner_size(size);

		let config_picker = |configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>| {
			configs
				.filter(|c| c.srgb_capable())
				.max_by_key(|c| c.num_samples())
				.expect("No GL configs found")
		};
		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template_builder, config_picker)
			.expect("Failed DisplayBuilder.build");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window
			.window_handle()
			.expect("Failed Window.window_handle")
			.as_raw();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe { gl_display.create_context(&gl_config, &context_attributes) }
			.expect("Failed Display.create_context");

		let surface_attributes_builder = SurfaceAttributesBuilder::<WindowSurface>::new()
			.with_srgb(Some(true));
		let surface_attributes = surface_attributes_builder.build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");

		let context = not_current.make_current(&surface)
			.expect("Failed NotCurrentContext.make_current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		GlWindow { size, window, surface, context }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
	}
}

struct RendererDemo {
	g: shade::gl::GlGraphics,
	scene: Scene,
}

impl RendererDemo {
	fn new(screen_size: Vec2i) -> RendererDemo {
		let mut g = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: false });

		let scene = {
			let epoch = time::Instant::now();

			let camera = {
				let pivot = Vec3f::ZERO;
				let position = Vec3f(0.0, -50.0, 30.0);
				shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
			};

			let shadow_map = shade::Texture2D::INVALID;

			let axes = {
				let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
				shade::d3::axes::AxesModel::create(&mut g, shader)
			};

			let cube = cube::Renderable::create(&mut g);
			let bunny = bunny::Renderable::create(&mut g);
			let color_tree = colortree::Renderable::create(&mut g);
			let oldtree = oldtree::Renderable::create(&mut g);
			let parallax = parallax::Renderable::create(&mut g);
			let globe = globe::Renderable::create(&mut g);

			Scene { screen_size, epoch, camera, shadow_map, axes, cube, bunny, color_tree, oldtree, parallax, globe }
		};

		RendererDemo { g, scene }
	}

	fn draw(&mut self) {
		self.scene.draw(&mut self.g);
	}
}

struct App {
	window: GlWindow,
	demo: RendererDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let screen_size = Vec2::new(window.size.width as i32, window.size.height as i32);
		let demo = RendererDemo::new(screen_size);
		Box::new(App { window, demo })
	}
	fn draw(&mut self) {
		self.demo.draw();
	}
}

//----------------------------------------------------------------

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
	let size = winit::dpi::PhysicalSize::new(800, 600);

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
					app = Some(App::new(event_loop, size));
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.window.resize(new_size);
						app.demo.scene.screen_size.x = new_size.width as i32;
						app.demo.scene.screen_size.y = new_size.height as i32;
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let dx = position.x as f32 - cursor_position.x as f32;
						let dy = position.y as f32 - cursor_position.y as f32;
						if left_click {
							auto_rotate = false;
							app.demo.scene.camera.rotate(-dx, -dy);
						}
						if right_click {
							auto_rotate = false;
							app.demo.scene.camera.pan(-dx, dy);
						}
						if middle_click {
							app.demo.scene.camera.zoom(dy * 0.01);
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
							app.demo.scene.camera.rotate(-1.0, 0.0);
						}
						app.draw();
						app.window.surface.swap_buffers(&app.window.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					app.window.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
