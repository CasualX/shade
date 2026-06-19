use crate::*;

struct ShaderToyUniforms {
	time: f32,
	aspect_ratio: f32,
}

impl shade::UniformVisitor for ShaderToyUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
		set.value("u_aspectRatio", &self.aspect_ratio);
	}
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

uniform float u_time;
uniform float u_aspectRatio;

#ifdef VERTEX_SHADER
void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	vec2 p=(v_uv-0.5)*3.0;
	p.x*=u_aspectRatio;
	vec4 o;
	vec2 l,v=p*(1.-(l+=abs(.7-dot(p,p))))/.2;
	for(float i;i++<8.;o+=(sin(v.xyyx)+1.)*abs(v.x-v.y)*.2)
		v+=cos(v.yx*i+vec2(0,i)+u_time)/i+.7;
	o_fragColor=tanh(exp(p.y*vec4(1,-1,-2,0))*exp(-4.*l.x)/o);
}
#endif
"#;

pub fn create(g: &mut shade::Graphics, _assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(ShaderToy::new(g))
}

struct ShaderToy {
	pp: shade::d2::PostProcessQuad,
	shadertoy: Box<dyn shade::ShaderProgram>,
}

impl ShaderToy {
	fn new(g: &mut shade::Graphics) -> ShaderToy {
		let pp = shade::d2::PostProcessQuad::create(g);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shadertoy = g.shader_compile(&mut source, "main.glsl", &[]);
		ShaderToy { pp, shadertoy }
	}
}

impl DemoInterface for ShaderToy {
	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		self.pp.draw(g, &*self.shadertoy, shade::BlendMode::Solid, &[&ShaderToyUniforms {
			time: frame.time as f32,
			aspect_ratio: frame.viewport.size().x as f32 / frame.viewport.size().y as f32,
		}]);
		g.end();
	}
}
