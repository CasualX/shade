use std::mem;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

const FRAGMENT_SHADER: &str = r#"
#version 330
out vec4 o_fragcolor;

in vec2 v_pos;

uniform sampler2D u_gradient;

float mandelbrot(vec2 c)
{
	const int max_iter = 100;
	vec2 z = vec2(0.0);
	int iter = 0;

	for (; iter < max_iter; ++iter) {
		if (dot(z, z) > 4.0) break;
		z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
	}

	return float(iter) / float(max_iter);

	// if (iter == max_iter) {
	// 	// Inside the set
	// 	return 1.0;
	// }

	// // Smooth escape time coloring
	// float zn = length(z);
	// float log_zn = log(zn * zn) / 2.0;
	// float nu = log(log_zn / log(2.0)) / log(2.0);
	// float escape = float(iter) + 1.0 - nu;

	// // Normalize and compress to [0,1)
	// float normalized = log(escape) / log(1.3);
	// return fract(normalized);
}

void main()
{
	float s = mandelbrot(v_pos);
	o_fragcolor = texture(u_gradient, vec2(s, 0.0));
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
in vec2 a_pos;

out vec2 v_pos;

uniform mat3x2 u_transform;

void main()
{
	v_pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

//----------------------------------------------------------------

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "a_pos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.position) as u16,
			},
		],
	};
}

static VERTICES: [Vertex; 6] = [
	Vertex { position: Vec2f(-1.0, -1.0) },
	Vertex { position: Vec2f( 1.0, -1.0) },
	Vertex { position: Vec2f( 1.0,  1.0) },
	Vertex { position: Vec2f(-1.0, -1.0) },
	Vertex { position: Vec2f( 1.0,  1.0) },
	Vertex { position: Vec2f(-1.0,  1.0) },
];

struct Uniforms {
	transform: Transform2f,
	gradient: shade::Texture2D,
}

impl shade::UniformVisitor for Uniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_gradient", &self.gradient);
	}
}

//----------------------------------------------------------------

#[derive(Clone, Debug)]
struct ZoomView {
	// Complex plane center
	center: Vec2f,
	// Height of the view
	height: f32,
}

static DEFAULT_VIEW: ZoomView = ZoomView {
	center: Vec2f::new(-0.5, 0.0),
	height: 2.0,
};

impl ZoomView {
	fn to_bounds(&self, aspect_ratio: f32) -> Bounds2f {
		let width = self.height * aspect_ratio;
		Bounds2::c(
			self.center.x - width / 2.0,
			self.center.y - self.height / 2.0,
			self.center.x + width / 2.0,
			self.center.y + self.height / 2.0,
		)
	}
}

#[derive(Default)]
struct ZoomViewStack {
	views: Vec<ZoomView>,
}

impl ZoomViewStack {
	fn current(&self) -> &ZoomView {
		self.views.last().unwrap_or(&DEFAULT_VIEW)
	}

	fn zoom(&mut self, pt: Vec2f, screen_size: Vec2f, factor: f32) {
		let current = self.current();

		let pt_frac = (pt - screen_size * 0.5) / screen_size.shuffle(Y, Y);
		let clicked_point = current.center + pt_frac * current.height;

		let center = clicked_point.lerp(current.center, factor);
		let height = current.height * factor;

		let next = ZoomView { center, height };
		self.views.push(next);
	}

	fn pan(&mut self, delta: Vec2f, screen_size: Vec2f) {
		// Pan start clones the current view so that we can modify it
		// This keeps the previous views intact for going back
		let Some(current) = self.views.last_mut() else { return };
		let delta_complex = delta / screen_size.shuffle(Y, Y) * current.height;
		current.center -= delta_complex;
	}

	fn back(&mut self) {
		self.views.pop();
	}
}

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	vb: shade::VertexBuffer,
	shader: shade::Shader,
	gradient: shade::Texture2D,
	pan_start: Point2f,
	panning: bool,
	cursor: Point2f,
	stack: ZoomViewStack,
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

		// Create the graphics context
		let mut g = shade::gl::GlGraphics::new();

		let vb = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static);
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		let gradient = {
			let gradient = shade::image::DecodedImage::load_file_png("examples/mandelbrot/gradient.png").unwrap();
			g.image(None, &gradient)
		};

		let pan_start = Point2f::ZERO;
		let panning = false;
		let cursor = Point2f::ZERO;
		let stack = ZoomViewStack::default();

		Box::new(App { size, window, surface, context, g, vb, shader, gradient, pan_start, panning, cursor, stack })
	}

	fn draw(&mut self) {
		// Render the frame
		let viewport = Bounds2::c(0, 0, self.size.width as i32, self.size.height as i32);
		self.g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		shade::clear!(self.g, color: Vec4(0.2, 0.5, 0.2, 1.0));

		let aspect_ratio = self.size.width as f32 / self.size.height as f32;

		// Compute the transform for the current zoom view
		let zoom_view = self.stack.current();
		let view_bounds = zoom_view.to_bounds(aspect_ratio);
		let transform = Transform2f::ortho(view_bounds).inverse();

		let uniforms = Uniforms { transform, gradient: self.gradient };

		self.g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vb,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[&uniforms],
			vertex_start: 0,
			vertex_end: 6,
			instances: -1,
		});

		self.g.end();
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let mut app: Option<Box<App>> = None;

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
						app.surface.resize(&app.context, width, height);
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.cursor.x = position.x as f32;
						app.cursor.y = position.y as f32;
						if app.panning {
							let size_vec = Vec2::new(app.size.width as f32, app.size.height as f32);
							app.stack.pan(app.cursor - app.pan_start, size_vec);
							app.pan_start = app.cursor;
							app.window.request_redraw();
						}
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						match button {
							MouseButton::Left => {
								if matches!(state, ElementState::Pressed) {
									let size_vec = Vec2::new(app.size.width as f32, app.size.height as f32);
									app.stack.zoom(app.cursor, size_vec, 0.5);
									app.window.request_redraw();
								}
							}
							MouseButton::Right => {
								if matches!(state, ElementState::Pressed) {
									app.pan_start = app.cursor;
									app.panning = true;
									app.stack.views.push(app.stack.current().clone());
								} else {
									app.panning = false;
								}
							}
							MouseButton::Middle => {
								if matches!(state, ElementState::Pressed) {
									app.stack.back();
									app.window.request_redraw();
								}
							}
							_ => {}
						}
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
			_ => {}
		}
	});
}
