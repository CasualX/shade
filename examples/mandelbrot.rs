use std::mem;
use cvmath::*;

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
	fn to_bounds(&self, aspect_ratio: f32) -> Bounds2<f32> {
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

fn main() {
	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_inner_size(size);

	let window_context = glutin::ContextBuilder::new()
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let mut g = shade::gl::GlGraphics::new();

	let vb = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static).unwrap();
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

	let gradient = shade::image::png::load_file(&mut g, None, "examples/mandelbrot/gradient.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	let mut pan_start = Point2f::ZERO;
	let mut panning = false;
	let mut cursor = Point2f::ZERO;
	let mut stack = ZoomViewStack::default();

	// Main loop
	event_loop.run(move |event, _, control_flow| {
		match event {
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
				*control_flow = winit::event_loop::ControlFlow::Exit
			}
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
				size = new_size;
				context.resize(new_size);
			}
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
				cursor.x = position.x as f32;
				cursor.y = position.y as f32;
				if panning {
					stack.pan(cursor - pan_start, Vec2::new(size.width as f32, size.height as f32));
					pan_start = cursor;
					context.window().request_redraw();
				}
			}
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button, .. }, .. } => {
				if matches!(button, winit::event::MouseButton::Left) {
					if matches!(state, winit::event::ElementState::Pressed) {
						stack.zoom(cursor, Vec2::new(size.width as f32, size.height as f32), 0.8);
						// request redraw
						context.window().request_redraw();
					}
				}
				else if matches!(button, winit::event::MouseButton::Right) {
					if matches!(state, winit::event::ElementState::Pressed) {
						pan_start = cursor;
						panning = true;
						// Panning modifies the current view
						stack.views.push(stack.current().clone());
					}
					else {
						panning = false;
					}
				}
				else if matches!(button, winit::event::MouseButton::Middle) {
					if matches!(state, winit::event::ElementState::Pressed) {
						stack.back();
						context.window().request_redraw();
					}
				}
			}
			winit::event::Event::RedrawRequested(_) => {
				// Render the frame
				g.begin().unwrap();

				g.clear(&shade::ClearArgs {
					surface: shade::Surface::BACK_BUFFER,
					color: Some(Vec4(0.2, 0.5, 0.2, 1.0)),
					..Default::default()
				}).unwrap();

				let aspect_ratio = size.width as f32 / size.height as f32;

				// Compute the transform for the current zoom view
				let zoom_view = stack.current();
				let view_bounds = zoom_view.to_bounds(aspect_ratio);
				let transform = Transform2f::ortho(view_bounds).inverse();

				let uniforms = Uniforms { transform, gradient };

				g.draw(&shade::DrawArgs {
					surface: shade::Surface::BACK_BUFFER,
					viewport: Bounds2::c(0, 0, size.width as i32, size.height as i32),
					scissor: None,
					blend_mode: shade::BlendMode::Solid,
					depth_test: None,
					cull_mode: None,
					mask: shade::DrawMask::COLOR,
					prim_type: shade::PrimType::Triangles,
					shader,
					vertices: &[shade::DrawVertexBuffer {
						buffer: vb,
						divisor: shade::VertexDivisor::PerVertex,
					}],
					uniforms: &[&uniforms],
					vertex_start: 0,
					vertex_end: 6,
					instances: -1,
				}).unwrap();

				g.end().unwrap();

				// Swap buffers
				context.swap_buffers().unwrap();
			}
			_ => (),
		}
	});
}
