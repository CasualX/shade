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
mod particles;

trait IRenderable {
	fn update(&mut self, globals: &Globals);
	fn draw(&self, g: &mut shade::Graphics, globals: &Globals, camera: &shade::d3::Camera, light: &Light, shadow: bool);
	fn get_bounds(&self) -> (Bounds3f, Transform3f);
}

struct Light {
	light_pos: Vec3f,
	light_view_proj: Mat4f,
	shadow_map: shade::Texture2D,
}
impl shade::UniformVisitor for Light {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_lightPos", &self.light_pos);
		set.value("u_shadowMap", &self.shadow_map);
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

struct RendererDemo {
	epoch: time::Instant,
	camera: shade::d3::ArcballCamera,
	shadow_map: shade::Texture2D,
	draw_bounds: bool,

	axes: shade::d3::axes::AxesModel,
	cube: cube::Renderable,
	bunny: bunny::Renderable,
	color_tree: colortree::Renderable,
	oldtree: oldtree::Renderable,
	parallax: parallax::Renderable,
	globe: globe::Renderable,
	particles: particles::Renderable,
	color3d_shader: shade::ShaderProgram,
}
impl RendererDemo {
	fn create(g: &mut shade::Graphics) -> RendererDemo {
		let epoch = time::Instant::now();

		let camera = {
			let pivot = Vec3f::ZERO;
			let position = Vec3f(0.0, -50.0, 30.0);
			shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
		};

		let shadow_map = shade::Texture2D::INVALID;
		let draw_bounds = false;

		let axes = {
			let shader = g.shader_compile(shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
			shade::d3::axes::AxesModel::create(g, shader)
		};

		let cube = cube::Renderable::create(g);
		let bunny = bunny::Renderable::create(g);
		let color_tree = colortree::Renderable::create(g);
		let oldtree = oldtree::Renderable::create(g);
		let parallax = parallax::Renderable::create(g);
		let globe = globe::Renderable::create(g);
		let particles = particles::Renderable::create(g);
		let color3d_shader = g.shader_compile(shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);

		RendererDemo {
			epoch,
			camera,
			shadow_map,
			draw_bounds,
			axes,
			cube,
			bunny,
			color_tree,
			oldtree,
			parallax,
			globe,
			particles,
			color3d_shader,
		}
	}
	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let time = self.epoch.elapsed().as_secs_f32();
		let globals = Globals { time };

		self.cube.update(&globals);
		self.particles.update_positions(g, &globals);

		let renderables = [
			&self.cube as &dyn IRenderable,
			&self.bunny as &dyn IRenderable,
			&self.color_tree as &dyn IRenderable,
			&self.oldtree as &dyn IRenderable,
			&self.parallax as &dyn IRenderable,
			&self.globe as &dyn IRenderable,
			&self.particles as &dyn IRenderable,
		];

		self.shadow_map = g.texture2d_update(self.shadow_map, &shade::Texture2DInfo {
			width: SHADOW_MAP_SIZE,
			height: SHADOW_MAP_SIZE,
			format: shade::TextureFormat::Depth32F,
			props: shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage!(SAMPLED | DEPTH_STENCIL_TARGET),
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Border,
				wrap_v: shade::TextureWrap::Border,
				compare: Some(shade::Compare::LessEqual),
				border_color: [1.0, 1.0, 1.0, 1.0],
				..Default::default()
			},
		});

		let mut light = Light {
			light_pos: animate_light_pos(time),
			light_view_proj: Mat4::IDENTITY,
			shadow_map: self.shadow_map,
		};

		// Render shadow map
		let shadow_viewport = Bounds2::vec(Vec2::dup(SHADOW_MAP_SIZE));
		g.begin(&shade::BeginArgs::Immediate {
			viewport: shadow_viewport,
			color: &[],
			levels: None,
			depth: self.shadow_map,
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
			shade::d3::Camera { viewport: shadow_viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		light.light_view_proj = light_camera.view_proj;

		shade::clear!(g, depth: 1.0);
		for &renderable in renderables.iter() {
			let (bounds, transform) = renderable.get_bounds();
			if light_camera.is_visible(&bounds, Some(&transform)) {
				renderable.draw(g, &globals, &light_camera, &light, true);
			}
		}
		g.end();

		// Render the frame
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.5, 0.2, 0.2, 1.0), depth: 1.0);

		// Camera setup
		let camera = {
			let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
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

		// Draw the scene
		let mut count = 0;
		for &renderable in renderables.iter() {
			let (bounds, transform) = renderable.get_bounds();
			if camera.is_visible(&bounds, Some(&transform)) {
				renderable.draw(g, &globals, &camera, &light, false);
				count += 1;
			}
		}
		print!("\rDrawn {} objects  ", count);

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::translate(self.camera.pivot) * Transform3f::scale(Vec3::dup(self.camera.pivot.distance(self.camera.position()) * 0.25)),
			depth_test: None,
		});

		if self.draw_bounds {
			let mut buf = shade::im::DrawBuilder::<shade::d3::ColorVertex3, shade::d3::ColorUniform3>::new();
			buf.shader = self.color3d_shader;
			buf.blend_mode = shade::BlendMode::Alpha;
			buf.depth_test = None;//Some(shade::DepthTest::GreaterEqual);
			buf.uniform.transform = camera.view_proj;
			buf.uniform.colormod = Vec4f(1.0, 1.0, 1.0, 1.0);
			for &renderable in renderables.iter() {
				draw_box(&mut buf, renderable);
			}
			buf.draw(g);
		}

		// Finish the frame
		g.end();
	}
}


fn draw_box(buf: &mut shade::im::DrawBuilder::<shade::d3::ColorVertex3, shade::d3::ColorUniform3>, renderable: &dyn IRenderable) {
	let mut p = buf.begin(shade::PrimType::Lines, 8, 12);

	static INDICES: [[bool; 3]; 8] = [
		[false, false, false],
		[false, false, true ],
		[false, true,  false],
		[false, true,  true ],
		[true,  false, false],
		[true,  false, true ],
		[true,  true,  false],
		[true,  true,  true ],
	];

	let (bounds, transform) = renderable.get_bounds();

	// Transform to clip space
	let pts: &[Vec3f; 2] = bounds.as_ref();
	let corners = INDICES.map(|[x, y, z]| shade::d3::ColorVertex3 {
		pos: transform * Vec3f(pts[x as usize].x, pts[y as usize].y, pts[z as usize].z),
		color: [255, 255, 0, 128],
	});

	p.add_vertices(&corners);

	let line_indices: &[u16] = &[
		0, 1, 0, 2, 0, 4,
		1, 3, 1, 5,
		2, 3, 2, 6,
		3, 7,
		4, 5, 4, 6,
		5, 7,
		6, 7,
	];
	p.add_indices(line_indices);
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

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: RendererDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = RendererDemo::create(opengl.as_graphics());
		Box::new(App { window, opengl, demo })
	}
	fn draw(&mut self) {
		let viewport = Bounds2i::c(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
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
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let dx = position.x as f32 - cursor_position.x as f32;
						let dy = position.y as f32 - cursor_position.y as f32;
						if left_click {
							auto_rotate = false;
							app.demo.camera.rotate(-dx, -dy);
						}
						if right_click {
							auto_rotate = false;
							app.demo.camera.pan(-dx, dy);
						}
						if middle_click {
							app.demo.camera.zoom(dy * 0.01);
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
							app.demo.camera.rotate(-1.0, 0.0);
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
