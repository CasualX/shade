use crate::*;

const FIELD_WIDTH: i32 = 256;
const FIELD_HEIGHT: i32 = 256;

const CONWAY_PROGRAM: &str = r#"
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

uniform sampler2D u_state;

int alive_at(ivec2 p, ivec2 size) {
	ivec2 q = ivec2((p.x % size.x + size.x) % size.x, (p.y % size.y + size.y) % size.y);
	return (texelFetch(u_state, q, 0).r > 0.5) ? 1 : 0;
}

#ifdef VERTEX_SHADER
void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	ivec2 size = textureSize(u_state, 0);
	ivec2 p = ivec2(gl_FragCoord.xy);
	int n = 0;
	n += alive_at(p + ivec2(-1, -1), size);
	n += alive_at(p + ivec2( 0, -1), size);
	n += alive_at(p + ivec2( 1, -1), size);
	n += alive_at(p + ivec2(-1,  0), size);
	n += alive_at(p + ivec2( 1,  0), size);
	n += alive_at(p + ivec2(-1,  1), size);
	n += alive_at(p + ivec2( 0,  1), size);
	n += alive_at(p + ivec2( 1,  1), size);
	int alive = alive_at(p, size);
	int next_alive = alive == 1 ? (((n == 2) || (n == 3)) ? 1 : 0) : ((n == 3) ? 1 : 0);
	float v = float(next_alive);
	o_fragColor = vec4(v, 0.0, 0.0, 1.0);
}
#endif
"#;

const DISPLAY_PROGRAM: &str = r#"
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

uniform sampler2D u_state;

#ifdef VERTEX_SHADER
void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	float v = texture(u_state, v_uv).r;
	o_fragColor = vec4(vec3(v), 1.0);
}
#endif
"#;

struct StateUniforms {
	state: shade::Texture2D,
}

impl shade::UniformVisitor for StateUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_state", &[self.state]);
	}
}

pub fn create(g: &mut shade::Graphics, _assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Conway::new(g))
}

struct Conway {
	pp: shade::d2::PostProcessQuad,
	conway_shader: shade::ShaderProgram,
	display_shader: shade::ShaderProgram,
	field_size: Vec2i,
	ping: usize,
	state: [shade::Texture2D; 2],
}

impl Conway {
	fn new(g: &mut shade::Graphics) -> Conway {
		let pp = shade::d2::PostProcessQuad::create(g);
		let mut source = shade::shader_interface! {
			files {
				"conway.glsl" => CONWAY_PROGRAM,
				"display.glsl" => DISPLAY_PROGRAM,
			}
		};
		let conway_shader = g.shader_compile(&mut source, "conway.glsl", &[]);
		let display_shader = g.shader_compile(&mut source, "display.glsl", &[]);
		let field_size = Vec2(FIELD_WIDTH, FIELD_HEIGHT);
		let seed = seed_data(field_size.x, field_size.y);
		let info = shade::Texture2DInfo {
			format: shade::TextureFormat::R8,
			width: field_size.x,
			height: field_size.y,
			props: shade::TextureProps! {
				usage: shade::TextureUsage!(WRITE | SAMPLED | COLOR_TARGET),
				filter: shade::TextureFilter::Nearest,
				wrap: shade::TextureWrap::Repeat,
			},
		};
		let state0 = g.texture2d(&info, &seed);
		let state1 = g.texture2d_create(&info);
		Conway { pp, conway_shader, display_shader, field_size, ping: 0, state: [state0, state1] }
	}

	fn step(&mut self, g: &mut shade::Graphics) {
		let src = self.state[self.ping];
		let dst = self.state[1 - self.ping];
		let viewport = Bounds2!(0, 0, self.field_size.x, self.field_size.y);
		g.begin(&shade::BeginArgs::Immediate {
			viewport,
			color: &[dst],
			levels: None,
			depth: shade::Texture2D::INVALID,
		});
		let uniforms = StateUniforms {
			state: src,
		};
		self.pp.draw(g, self.conway_shader, shade::BlendMode::Solid, &[&uniforms]);
		g.end();
		self.ping = 1 - self.ping;
	}
}

impl DemoInterface for Conway {
	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		self.step(g);
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.0, 0.0, 0.0, 1.0));
		let uniforms = StateUniforms {
			state: self.state[self.ping],
		};
		self.pp.draw(g, self.display_shader, shade::BlendMode::Solid, &[&uniforms]);
		g.end();
	}
}

fn seed_data(width: i32, height: i32) -> Vec<u8> {
	let mut rng = urandom::new();
	let mut data = vec![0u8; (width as usize) * (height as usize)];
	let x0 = width / 4;
	let x1 = (width * 3) / 4;
	let y0 = height / 4;
	let y1 = (height * 3) / 4;
	for y in y0..y1 {
		for x in x0..x1 {
			let i = (y as usize) * (width as usize) + (x as usize);
			data[i] = if rng.next::<u8>() < 51 { 255 } else { 0 };
		}
	}
	data
}
