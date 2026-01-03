use std::mem;
use shade::cvmath::*;

mod api;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec2f,
	uv: Vec2f,
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
			shade::VertexAttribute {
				name: "a_uv",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.uv) as u16,
			},
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
precision highp float;

varying vec2 v_uv;

uniform sampler2D u_texture;
uniform sampler2D u_displacement;
uniform float u_time;
uniform vec3 u_waterbase;
uniform vec3 u_wavehighlight;
uniform vec3 u_waveshadow;

void main() {
	vec2 uv = v_uv;

	// Layered distortion
	float distortion1 = texture2D(u_displacement, fract(uv * 1.0 + vec2(u_time * 0.2, u_time * 0.25))).r;
	float distortion2 = texture2D(u_displacement, fract(uv * 2.5 + vec2(-u_time * 0.15, u_time * 0.1))).r;
	float distortion3 = texture2D(u_displacement, fract(uv * 0.75 + vec2(u_time * 0.05, -u_time * 0.2))).r;

	float distortion = (distortion1 * 0.5 + distortion2 * 0.3 + distortion3 * 0.2) - 0.5;
	uv += vec2(distortion * 0.1, distortion * 0.15);

	// Main wave layer
	float v = texture2D(u_texture, uv * vec2(4.0, 4.0)).r;
	vec3 mainColor = mix(vec3(0.0, 0.0, 0.5), vec3(0.5, 0.8, 1.0), v);

	// Shadow wave layer
	float u = texture2D(u_texture, uv * vec2(2.0, 2.0) + vec2(0.3, 0.3) + vec2(u_time * 0.05, -u_time * 0.05)).r;
	vec3 shadowColor = mix(vec3(0.0, 0.0, 0.0), u_waveshadow, u);

	vec3 finalColor = mainColor - shadowColor * 0.5;

	gl_FragColor = vec4(finalColor, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
attribute vec2 a_pos;
attribute vec2 a_uv;

varying vec2 v_uv;

void main()
{
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

#[derive(Clone, Debug)]
struct Uniform {
	time: f32,
	texture: shade::Texture2D,
	distortion: shade::Texture2D,
	waterbase: Vec3f,
	wavehighlight: Vec3f,
	waveshadow: Vec3f,
}

impl shade::UniformVisitor for Uniform {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
		set.value("u_texture", &self.texture);
		set.value("u_displacement", &self.distortion);
		set.value("u_waterbase", &self.waterbase);
		set.value("u_wavehighlight", &self.wavehighlight);
		set.value("u_waveshadow", &self.waveshadow);
	}
}

//----------------------------------------------------------------

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	shader: shade::Shader,
	texture: shade::Texture2D,
	distortion: shade::Texture2D,
	vb: shade::VertexBuffer,
	ib: shade::IndexBuffer,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();

		let g = shade::Graphics(&mut webgl);

		// Create the triangle shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		let texture = {
			let file_png = include_bytes!("../../../zeldawater/water.png");
			let image = shade::image::DecodedImage::load_memory_png(file_png).unwrap();
			let props = shade::TextureProps {
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			};
			g.image(None, &(&image, &props))
		};

		let distortion = {
			let file_png = include_bytes!("../../../zeldawater/distort.png");
			let image = shade::image::DecodedImage::load_memory_png(file_png).unwrap();
			let props = shade::TextureProps {
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			};
			g.image(None, &(&image, &props))
		};

		// Create the full screen quad vertex buffer
		let vb = g.vertex_buffer(None, &[
			Vertex { position: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
			Vertex { position: Vec2f(1.0, -1.0), uv: Vec2f(1.0, 0.0) },
			Vertex { position: Vec2f(-1.0, 1.0), uv: Vec2f(0.0, 1.0) },
			Vertex { position: Vec2f(1.0, 1.0), uv: Vec2f(1.0, 1.0) },
		], shade::BufferUsage::Static);

		let ib = g.index_buffer(None, &[
			0u16, 1, 2,
			1, 3, 2,
		], 4, shade::BufferUsage::Static);

		let screen_size = Vec2::ZERO;
		Context { webgl, screen_size, shader, texture, distortion, vb, ib }
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn draw(&mut self, time: f64) {
		let g = shade::Graphics(&mut self.webgl);
		// Render the frame
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		// Clear the screen
		g.clear(&shade::ClearArgs {
			color: Some(Vec4(0.2, 0.5, 0.2, 1.0)),
			..Default::default()
		});

		let time = time as f32;
		let texture = self.texture;
		let distortion = self.distortion;
		let waterbase = Vec3f(0.0, 0.0, 0.5);
		let wavehighlight = Vec3f(0.5, 0.8, 1.0);
		let waveshadow = Vec3f(0.1, 0.2, 0.3);
		let uniform = Uniform { time, texture, distortion, waterbase, wavehighlight, waveshadow };

		// Draw the quad
		g.draw_indexed(&shade::DrawIndexedArgs {
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
			indices: self.ib,
			uniforms: &[&uniform],
			index_start: 0,
			index_end: 6,
			instances: -1,
		});

		// Finish rendering
		g.end();
	}
}
