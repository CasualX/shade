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

impl ZoomView {
	fn to_bounds(&self, aspect_ratio: f32) -> Bounds2f {
		let width = self.height * aspect_ratio;
		Bounds2!(
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
		self.views.push(ZoomView { center, height: current.height * factor });
	}

	fn pan(&mut self, delta: Vec2f, screen_size: Vec2f) {
		let Some(current) = self.views.last_mut() else { return };
		let delta_complex = delta / screen_size.shuffle(Y, Y) * current.height;
		current.center -= delta_complex;
	}

	fn back(&mut self) {
		self.views.pop();
	}
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Mandelbrot::new(g, assets))
}

struct Mandelbrot {
	vertices: shade::VertexBuffer,
	shader: shade::ShaderProgram,
	gradient: shade::Texture2D,
	pan_start: Point2f,
	panning: bool,
	cursor: Point2f,
	screen_size: Vec2f,
	stack: ZoomViewStack,
}

impl Mandelbrot {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Mandelbrot {
		let vertices = g.vertex_buffer(&VERTICES, shade::BufferUsage::Static);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let bytes = assets.read("mandelbrot/gradient.png").unwrap();
		let gradient = shade::image::DecodedImage::load_memory_png(&bytes).unwrap();
		let gradient = g.image(&gradient);
		Mandelbrot {
			vertices,
			shader,
			gradient,
			pan_start: Point2f::ZERO,
			panning: false,
			cursor: Point2f::ZERO,
			screen_size: Vec2(1.0, 1.0),
			stack: ZoomViewStack::default(),
		}
	}
}

impl DemoInterface for Mandelbrot {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn resize(&mut self, size: Vec2i) {
		self.screen_size = Vec2(size.x.max(1) as f32, size.y.max(1) as f32);
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position, .. } => {
				self.cursor = Point2(position.x, position.y);
				if self.panning {
					self.stack.pan(self.cursor - self.pan_start, self.screen_size);
					self.pan_start = self.cursor;
					shell.request_redraw();
				}
			}
			Input::MouseButton { button: MouseButton::Left, pressed: true, .. } => {
				self.stack.zoom(self.cursor, self.screen_size, 0.5);
				shell.request_redraw();
			}
			Input::MouseButton { button: MouseButton::Right, pressed, .. } => {
				if pressed {
					self.pan_start = self.cursor;
					self.panning = true;
					self.stack.views.push(self.stack.current().clone());
				}
				else {
					self.panning = false;
				}
			}
			Input::MouseButton { button: MouseButton::Middle, pressed: true, .. } => {
				self.stack.back();
				shell.request_redraw();
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.2, 0.5, 0.2, 1.0));
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let view_bounds = self.stack.current().to_bounds(aspect_ratio);
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
		g.end();
	}
}
