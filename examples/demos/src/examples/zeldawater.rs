use crate::*;

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

const PROGRAM: &str = r#"
#version unified 330 core, 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec2 a_pos;
in vec2 a_uv;
#endif

VARYING vec2 v_uv;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

uniform sampler2D u_texture;
uniform sampler2D u_displacement;
uniform float u_time;
uniform vec3 u_waterbase;
uniform vec3 u_wavehighlight;
uniform vec3 u_waveshadow;

#ifdef VERTEX_SHADER
void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	vec2 uv = v_uv;
	float distortion1 = texture(u_displacement, fract(uv * 1.0 + vec2(u_time * 0.2, u_time * 0.25))).r;
	float distortion2 = texture(u_displacement, fract(uv * 2.5 + vec2(-u_time * 0.15, u_time * 0.1))).r;
	float distortion3 = texture(u_displacement, fract(uv * 0.75 + vec2(u_time * 0.05, -u_time * 0.2))).r;
	float distortion = (distortion1 * 0.5 + distortion2 * 0.3 + distortion3 * 0.2) - 0.5;
	uv += vec2(distortion * 0.1, distortion * 0.15);

	float v = texture(u_texture, uv * vec2(4.0, 4.0)).r;
	vec3 mainColor = mix(vec3(0.0, 0.0, 0.5), vec3(0.5, 0.8, 1.0), v);

	float u = texture(u_texture, uv * vec2(2.0, 2.0) + vec2(0.3, 0.3) + vec2(u_time * 0.05, -u_time * 0.05)).r;
	vec3 shadowColor = mix(vec3(0.0, 0.0, 0.0), u_waveshadow, u);

	vec3 finalColor = mainColor - shadowColor * 0.5;
	o_fragColor = vec4(finalColor, 1.0);
}
#endif
"#;

struct Uniform<'a> {
	time: f32,
	texture: &'a dyn shade::Texture2D,
	distortion: &'a dyn shade::Texture2D,
	waterbase: Vec3f,
	wavehighlight: Vec3f,
	waveshadow: Vec3f,
}

impl<'a> shade::UniformVisitor for Uniform<'a> {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
		set.value("u_texture", self.texture);
		set.value("u_displacement", self.distortion);
		set.value("u_waterbase", &self.waterbase);
		set.value("u_wavehighlight", &self.wavehighlight);
		set.value("u_waveshadow", &self.waveshadow);
	}
}

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(ZeldaWater::new(g, assets))
}

struct ZeldaWater {
	vertices: Box<dyn shade::VertexBuffer>,
	indices: Box<dyn shade::IndexBuffer>,
	texture: Box<dyn shade::Texture2D>,
	distortion: Box<dyn shade::Texture2D>,
	shader: Box<dyn shade::ShaderProgram>,
}

impl ZeldaWater {
	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> ZeldaWater {
		let vertices = g.vertex_buffer(&[
				Vertex { position: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
				Vertex { position: Vec2f(1.0, -1.0), uv: Vec2f(1.0, 0.0) },
				Vertex { position: Vec2f(-1.0, 1.0), uv: Vec2f(0.0, 1.0) },
				Vertex { position: Vec2f(1.0, 1.0), uv: Vec2f(1.0, 1.0) },
			],
			shade::BufferUsage::Static,
		);
		let indices = g.index_buffer(&[0u16, 1, 2, 1, 3, 2], 4, shade::BufferUsage::Static);
		let texture = Self::load_repeat_texture(g, assets, "zeldawater/water.png");
		let distortion = Self::load_repeat_texture(g, assets, "zeldawater/distort.png");
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		ZeldaWater { vertices, indices, texture, distortion, shader }
	}

	fn load_repeat_texture(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader, path: &str) -> Box<dyn shade::Texture2D> {
		let bytes = assets.read(path).unwrap();
		let image = shade::image::DecodedImage::load_memory_png(&bytes).unwrap();
		let props = shade::TextureProps! {
			usage: shade::TextureUsage::TEXTURE,
			filter: shade::TextureFilter::Linear,
			wrap: shade::TextureWrap::Repeat,
		};
		g.image(&props.bind(&image))
	}
}

impl DemoInterface for ZeldaWater {
	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.2, 0.5, 0.2, 1.0));
		let uniform = Uniform {
			time: frame.time as f32,
			texture: &*self.texture,
			distortion: &*self.distortion,
			waterbase: Vec3f(0.0, 0.0, 0.5),
			wavehighlight: Vec3f(0.5, 0.8, 1.0),
			waveshadow: Vec3f(0.1, 0.2, 0.3),
		};
		g.draw_indexed(&shade::DrawIndexedArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader: &*self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: &*self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: &*self.indices,
			uniforms: &[&uniform],
			index_start: 0,
			index_end: 6,
			instances: -1,
		});
		g.end();
	}
}
