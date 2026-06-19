use crate::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MyVertex3 {
	position: Vec3f,
	tex_coord: Vec2f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for MyVertex3 {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<MyVertex3>() as u16,
		alignment: mem::align_of::<MyVertex3>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(MyVertex3.position)),
			shade::VertexAttribute::with::<Vec2f>("a_uv", dataview::offset_of!(MyVertex3.tex_coord)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(MyVertex3.color)),
		],
	};
}

const PROGRAM: &str = r#"
#version unified 330 core, 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec3 a_pos;
in vec2 a_uv;
in vec4 a_color;
#endif

VARYING vec4 v_color;
VARYING vec2 v_uv;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

uniform mat4x4 u_transform;
uniform sampler2D u_texture;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

#ifdef VERTEX_SHADER
void main() {
	v_color = srgbToLinear(a_color);
	v_uv = a_uv;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	ivec2 texSize = textureSize(u_texture, 0);
	vec4 color = texture(u_texture, v_uv / vec2(texSize)) * v_color;
	if (color.a < 0.5) {
		discard;
	}
	color.a = 1.0;
	o_fragColor = color;
}
#endif
"#;

#[derive(Clone, Debug, PartialEq)]
struct MyUniform3<'a> {
	transform: Mat4f,
	texture: &'a dyn shade::Texture2D,
}

impl<'a> Default for MyUniform3<'a> {
	fn default() -> Self {
		MyUniform3 {
			transform: Mat4::IDENTITY,
			texture: &shade::DefaultTexture2D,
		}
	}
}

impl<'a> shade::UniformVisitor for MyUniform3<'a> {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", self.texture);
	}
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Scene::new(g, assets))
}

struct Scene {
	texture: Box<dyn shade::Texture2D>,
	shader: Box<dyn shade::ShaderProgram>,
}

impl Scene {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Scene {
		let props = shade::TextureProps! {
			usage: shade::TextureUsage::TEXTURE,
			filter: shade::TextureFilter::Nearest,
			wrap: shade::TextureWrap::Edge,
		};
		let texture = {
			let bytes = assets.read("textures/scene tiles.png").unwrap();
			let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
			g.image(&props.bind(&image))
		};
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		Scene { texture, shader }
	}
}

impl DemoInterface for Scene {
	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		let curtime = frame.time as f32;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.2, 0.2, 0.5, 1.0), depth: 1.0);
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let projection = Mat4::perspective(Angle::deg(45.0), aspect_ratio, 0.1, 1000.0, (Hand::RH, Clip::NO));
		let view = {
			let eye = Vec3(32.0 + (curtime * 2.0).sin() * 32.0, 100.0 + (curtime * 1.5).sin() * 32.0, -100.0) * 1.5;
			let target = Vec3(96.0 * 0.5, 0.0, 32.0);
			Transform3f::look_at(eye, target, Vec3(0.0, 1.0, 0.0), Hand::RH)
		};
		let mut cv = shade::im::DrawBuilder::<MyVertex3, MyUniform3>::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.depth_test = Some(shade::Compare::Less);
		cv.shader = Some(&*self.shader);
		cv.uniform.transform = projection * view;
		cv.uniform.texture = &*self.texture;
		floor_tile(&mut cv, 0, 0, &GRASS);
		floor_tile(&mut cv, 1, 0, &GRASS);
		floor_tile(&mut cv, 2, 0, &GRASS);
		floor_tile(&mut cv, 0, 1, &GRASS);
		floor_tile(&mut cv, 1, 1, &GRASS);
		floor_tile(&mut cv, 2, 1, &GRASS);
		floor_thing(&mut cv, 0, 0, &DROP);
		floor_thing(&mut cv, 1, 1, &DROP);
		floor_thing(&mut cv, 2, 0, &DROP);
		floor_thing(&mut cv, 1, 0, &BEAR);
		cv.draw(g);
		g.end();
	}
}

struct Sprite {
	left: f32,
	up: f32,
	right: f32,
	down: f32,
}

const GRASS: Sprite = Sprite { left: 1.0, up: 35.0, right: 32.0, down: 66.0 };
const DROP: Sprite = Sprite { left: 35.0, up: 35.0, right: 66.0, down: 66.0 };
const BEAR: Sprite = Sprite { left: 3.0, up: 68.0, right: 49.0, down: 152.0 };

fn floor_tile(cv: &mut shade::im::DrawBuilder<'_, MyVertex3, MyUniform3<'_>>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: Vec2(sprite.left, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: Vec2(sprite.left, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3((x + 1) as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: Vec2(sprite.right, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3((x + 1) as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: Vec2(sprite.right, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
}

fn floor_thing(cv: &mut shade::im::DrawBuilder<'_, MyVertex3, MyUniform3<'_>>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	let yoffs = -7.0;
	let zoffs1 = 12.0;
	let zoffs2 = 24.0;
	let width = sprite.right - sprite.left;
	let height = sprite.down - sprite.up;
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: Vec2(sprite.left, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: Vec2(sprite.left, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0 + width, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: Vec2(sprite.right, sprite.up),
		color: shade::norm!([255, 255, 255, 255]),
	});
	cv.add_vertex(MyVertex3 {
		position: Vec3(x as f32 * 32.0 + width, yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: Vec2(sprite.right, sprite.down),
		color: shade::norm!([255, 255, 255, 255]),
	});
}
