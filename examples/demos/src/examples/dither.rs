use crate::*;

struct PostProcessDitherUniforms {
	texture: shade::Texture2D,
	dither: shade::Texture2D,
	dither_scale: f32,
	levels: f32,
}

impl shade::UniformVisitor for PostProcessDitherUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
		set.sampler2d("u_dither", &[self.dither]);
		set.value("u_dither_scale", &self.dither_scale);
		set.value("u_levels", &self.levels);
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
out vec4 frag_color;
#endif

uniform sampler2D u_texture;
uniform sampler2D u_dither;
uniform float u_dither_scale;
uniform float u_levels;

#ifdef VERTEX_SHADER
void main() {
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	ivec2 pixel_pos = ivec2(gl_FragCoord.xy);
	ivec2 dither_size = textureSize(u_dither, 0);
	ivec2 dither_pos = ivec2(floor(vec2(pixel_pos) / max(u_dither_scale, 0.0001))) % dither_size;
	float t = texelFetch(u_dither, dither_pos, 0).r;
	float threshold = (t * 255.0 + 0.5) / 256.0;
	vec4 color = texture(u_texture, v_uv);
	float levels = max(u_levels, 2.0);
	vec3 value = color.rgb * (levels - 1.0);
	vec3 quant = floor(value + threshold);
	vec3 out_rgb = clamp(quant / (levels - 1.0), 0.0, 1.0);
	frag_color = vec4(out_rgb, color.a);
}
#endif
"#;

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Dither::new(g, assets))
}

struct Dither {
	pp: shade::d2::PostProcessQuad,
	pp_shader: shade::ShaderProgram,
	texture: shade::Texture2D,
	dither: [shade::Texture2D; 7],
	dither_index: usize,
	levels: f32,
}

impl Dither {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Dither {
		let dither = [
			g.image(&shade::dither::BAYER2x2),
			g.image(&shade::dither::BAYER4x4),
			g.image(&shade::dither::BAYER8x8),
			g.image(&shade::dither::BAYER16x16),
			g.image(&shade::dither::HALFTONE16x16),
			g.image(&shade::dither::BNVC32x32),
			g.image(&shade::dither::BNVC64x64),
		];
		let texture = {
			let bytes = assets.read("screenmelt/main-menu.png").unwrap();
			let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
			g.image(&image)
		};
		let pp = shade::d2::PostProcessQuad::create_flipped(g);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let pp_shader = g.shader_compile(&mut source, "main.glsl", &[]);
		Dither { pp, pp_shader, texture, dither, dither_index: 0, levels: 3.0 }
	}

	fn next_index(&mut self, offset: isize) {
		let len = self.dither.len() as isize;
		self.dither_index = (self.dither_index as isize + offset).rem_euclid(len) as usize;
	}

	fn adjust_levels(&mut self, delta: f32) {
		self.levels = (self.levels + delta).max(2.0);
	}
}

impl DemoInterface for Dither {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::KeyDown(Key::ArrowLeft) => self.next_index(-1),
			Input::KeyDown(Key::ArrowRight) => self.next_index(1),
			Input::KeyDown(Key::ArrowUp) => self.adjust_levels(1.0),
			Input::KeyDown(Key::ArrowDown) => self.adjust_levels(-1.0),
			_ => return,
		}
		shell.request_redraw();
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		let index = self.dither_index;
		let uniforms = PostProcessDitherUniforms {
			texture: self.texture,
			dither: self.dither[index],
			dither_scale: 1.0,
			levels: self.levels,
		};
		self.pp.draw(g, self.pp_shader, shade::BlendMode::Alpha, &[&uniforms]);
		g.end();
	}
}
