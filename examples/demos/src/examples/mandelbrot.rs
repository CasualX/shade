use crate::*;

const PROGRAM: &str = r#"
#version unified 330 core, 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec2 a_pos;
#endif

VARYING vec2 v_pos;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

uniform mat3x2 u_transform;
uniform sampler2D u_gradient;

float mandelbrot(vec2 c) {
	const int maxIter = 100;
	vec2 z = vec2(0.0);
	int iter = 0;
	for (; iter < maxIter; ++iter) {
		if (dot(z, z) > 4.0) break;
		z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
	}
	return float(iter) / float(maxIter);
}

#ifdef VERTEX_SHADER
void main() {
	v_pos = u_transform * vec3(a_pos, 1.0);
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	float s = mandelbrot(v_pos);
	o_fragColor = texture(u_gradient, vec2(s, 0.0));
}
#endif
"#;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[shade::VertexAttribute {
			name: "a_pos",
			format: shade::VertexAttributeFormat::F32v2,
			offset: dataview::offset_of!(Vertex.position) as u16,
		}],
	};
}

static VERTICES: [Vertex; 6] = [
	Vertex { position: Vec2f(-1.0, -1.0) },
	Vertex { position: Vec2f(1.0, -1.0) },
	Vertex { position: Vec2f(1.0, 1.0) },
	Vertex { position: Vec2f(-1.0, -1.0) },
	Vertex { position: Vec2f(1.0, 1.0) },
	Vertex { position: Vec2f(-1.0, 1.0) },
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

#[derive(Clone, Debug)]
struct ZoomView {
	center: Vec2f,
	height: f32,
}

static DEFAULT_VIEW: ZoomView = ZoomView {
	center: Vec2f::new(-0.5, 0.0),
	height: 2.0,
};

const MIN_SELECTION_SIZE: f32 = 8.0;
const ZOOM_ANIMATION_SECONDS: f32 = 0.22;
const MIN_VIEW_HEIGHT: f32 = 1.0e-12;
const ZOOM_ANIMATION_IDLE: f64 = -1.0;

impl ZoomView {
	fn to_bounds(&self, aspect_ratio: f32) -> Bounds2f {
		Bounds2::point(self.center, Vec2(aspect_ratio, 1.0) * self.height * 0.5)
	}

	fn from_bounds(bounds: Bounds2f) -> ZoomView {
		let bounds = bounds.norm();
		ZoomView {
			center: bounds.center(),
			height: bounds.height().max(MIN_VIEW_HEIGHT),
		}
	}
}

struct ZoomViewStack {
	current: ZoomView,
	previous: ZoomView,
	history: Vec<ZoomView>,
	animation_start: f64,
}

impl ZoomViewStack {
	fn new() -> ZoomViewStack {
		ZoomViewStack {
			current: DEFAULT_VIEW.clone(),
			previous: DEFAULT_VIEW.clone(),
			history: Vec::new(),
			animation_start: ZOOM_ANIMATION_IDLE,
		}
	}

	fn current(&self) -> &ZoomView {
		&self.current
	}

	fn screen_to_view(&self, pt: Vec2f, screen_size: Vec2f) -> Vec2f {
		let current = self.current();
		let aspect_ratio = screen_size.x / screen_size.y;
		let bounds = current.to_bounds(aspect_ratio);
		bounds.mins + pt / screen_size * bounds.size()
	}

	fn zoom_rect(&mut self, start: Vec2f, end: Vec2f, screen_size: Vec2f, time: f64) -> bool {
		let screen_bounds = Bounds2(start, end).norm();
		if screen_bounds.size().vmin() < MIN_SELECTION_SIZE {
			return false;
		}

		let view_bounds = Bounds2(
			self.screen_to_view(screen_bounds.mins, screen_size),
			self.screen_to_view(screen_bounds.maxs, screen_size),
		).norm();
		let view_size = view_bounds.size();
		let screen_aspect = screen_size.x / screen_size.y;
		let zoom_height = view_size.y.max(view_size.x / screen_aspect);
		self.history.push(self.current.clone());
		self.previous = self.current.clone();
		self.current = ZoomView {
			center: view_bounds.center(),
			height: zoom_height,
		};
		self.begin_animation(time);
		return true;
	}

	fn pan(&mut self, delta: Vec2f, screen_size: Vec2f) {
		let pan_fraction = delta / screen_size.shuffle(Y, Y);
		let current_delta = pan_fraction * self.current.height;
		let previous_delta = pan_fraction * self.previous.height;
		self.current.center -= current_delta;
		self.previous.center -= previous_delta;
	}

	fn back(&mut self, time: f64) {
		self.previous = self.current.clone();
		self.current = self.history.pop().unwrap_or_else(|| DEFAULT_VIEW.clone());
		self.begin_animation(time);
	}

	fn begin_animation(&mut self, time: f64) {
		let center_is_finite = self.current.center.map(f32::is_finite).reduce(|a, b| a && b);
		if !center_is_finite || !self.current.height.is_finite() || self.current.height <= 0.0 {
			self.current = self.previous.clone();
			self.animation_start = ZOOM_ANIMATION_IDLE;
			return;
		}
		self.current.height = self.current.height.max(MIN_VIEW_HEIGHT);
		self.animation_start = time;
	}

	fn view(&mut self, time: f64, aspect_ratio: f32) -> ZoomView {
		if !self.is_animating() {
			return self.current.clone();
		}

		let elapsed = self.animation_elapsed(time);
		if elapsed >= ZOOM_ANIMATION_SECONDS {
			self.previous = self.current.clone();
			self.animation_start = ZOOM_ANIMATION_IDLE;
			return self.current.clone();
		}

		let t = (elapsed / ZOOM_ANIMATION_SECONDS).clamp(0.0, 1.0);
		let t = ease_out_cubic(t);

		let from = self.previous.to_bounds(aspect_ratio);
		let to = self.current.to_bounds(aspect_ratio);
		ZoomView::from_bounds(lerp(from, to, t))
	}

	fn animation_elapsed(&self, time: f64) -> f32 {
		return (time - self.animation_start).max(0.0) as f32;
	}

	fn is_animating(&self) -> bool {
		self.animation_start != ZOOM_ANIMATION_IDLE
	}
}

fn ease_out_cubic(t: f32) -> f32 {
	1.0 - (1.0 - t).powi(3)
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Mandelbrot::new(g, assets))
}

struct Mandelbrot {
	vertices: shade::VertexBuffer,
	shader: shade::ShaderProgram,
	gradient: shade::Texture2D,
	color_shader: shade::ShaderProgram,
	cursor: Point2f,
	drag_selection: Option<Bounds2f>,
	panning: bool,
	screen_size: Vec2f,
	stack: ZoomViewStack,
}

impl Mandelbrot {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Mandelbrot {
		let vertices = g.vertex_buffer(&VERTICES, shade::BufferUsage::Static);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let color_shader = g.shader_compile(&mut source, "color.glsl", &[]);
		let bytes = assets.read("mandelbrot/gradient.png").unwrap();
		let gradient = shade::image::DecodedImage::load_memory_png(&bytes).unwrap();
		let gradient = g.image(&gradient);
		Mandelbrot {
			vertices,
			shader,
			gradient,
			color_shader,
			cursor: Point2f::ZERO,
			drag_selection: None,
			panning: false,
			screen_size: Vec2(1.0, 1.0),
			stack: ZoomViewStack::new(),
		}
	}

	fn drag_view_bounds(&self) -> Option<Bounds2f> {
		let bounds = self.drag_selection?;
		Some(Bounds2(
			self.stack.screen_to_view(bounds.mins, self.screen_size),
			self.stack.screen_to_view(bounds.maxs, self.screen_size),
		).norm())
	}
}

impl DemoInterface for Mandelbrot {
	fn redraw_mode(&self) -> RedrawMode {
		if self.stack.is_animating() {
			RedrawMode::Continuous
		}
		else {
			RedrawMode::OnDemand
		}
	}

	fn resize(&mut self, size: Vec2i) {
		self.screen_size = size.cast();
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position, .. } => {
				let prev_cursor = self.cursor;
				self.cursor = Point2(position.x, position.y);
				if let Some(selection) = &mut self.drag_selection {
					selection.maxs = self.cursor;
					shell.request_redraw();
				}
				if self.panning {
					self.stack.pan(self.cursor - prev_cursor, self.screen_size);
					shell.request_redraw();
				}
			}
			Input::MouseButton { button: MouseButton::Left, pressed, position } => {
				self.cursor = position;
				if pressed {
					self.drag_selection = Some(Bounds2(self.cursor, self.cursor));
				}
				else if let Some(selection) = self.drag_selection.take() {
					if self.stack.zoom_rect(selection.mins, selection.maxs, self.screen_size, shell.get_time()) {
						shell.request_redraw();
					}
				}
			}
			Input::MouseButton { button: MouseButton::Right, pressed, position } => {
				self.cursor = position;
				self.panning = pressed;
			}
			Input::MouseButton { button: MouseButton::Middle, pressed: true, .. } => {
				self.stack.back(shell.get_time());
				shell.request_redraw();
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.2, 0.5, 0.2, 1.0));
		let view_bounds = self.stack.view(frame.time, aspect_ratio).to_bounds(aspect_ratio);
		let uniforms = Uniforms {
			transform: Transform2f::ortho(view_bounds).inverse(),
			gradient: self.gradient,
		};
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[&uniforms],
			vertex_start: 0,
			vertex_end: 6,
			instances: -1,
		});
		if let Some(bounds) = self.drag_view_bounds() {
			let mut buf = d2::ColorBuffer::new();
			buf.blend_mode = shade::BlendMode::Alpha;
			buf.cull_mode = None;
			buf.uniform.transform = Transform2f::ortho(view_bounds);
			buf.shader = self.color_shader;
			let color = Vec4(255, 255, 255, 230);
			let pen = d2::Pen {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			buf.draw_line_rect(&pen, &bounds);
			buf.draw(g);
		}
		g.end();
	}
}
