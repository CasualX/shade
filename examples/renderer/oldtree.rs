use cvmath::*;

const COMMON: &str = r#"
#ifdef VERTEX_SHADER
in vec3 a_pos;
in vec3 a_normal;
in vec2 a_uv;
#endif

VARYING vec3 v_fragPos;
VARYING vec3 v_normal;
VARYING vec2 v_uv;
VARYING vec4 v_lightClip;

uniform mat4x3 u_model;
uniform mat4 u_viewProjMatrix;
uniform mat4 u_lightTransform;

#ifdef VERTEX_SHADER
void main() {
	v_fragPos = vec3(u_model * vec4(a_pos, 1.0));
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;
	v_uv = a_uv;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_viewProjMatrix * vec4(v_fragPos, 1.0);
}
#endif
"#;

const PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
uniform sampler2D u_diffuse;
uniform vec3 u_cameraPosition;
uniform vec3 u_lightPos;
uniform sampler2DShadow u_shadowMap;

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	vec3 norm = normalize(v_normal);
	float diff = max(dot(norm, lightDir), 0.0);
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	vec4 texColor = texture(u_diffuse, uv);
	if (texColor.a < 0.1) {
		discard;
	}
	vec3 lightNdc = v_lightClip.xyz / v_lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float bias = 0.001;
	float visibility = texture(u_shadowMap, vec3(shadowUvZ.xy, shadowUvZ.z - bias));
	vec3 finalColor = texColor.rgb * (0.4 + visibility * (diff * 0.8));
	o_fragColor = vec4(finalColor, texColor.a);
}
#endif
"#;

const SHADOW_PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
uniform sampler2D u_diffuse;

void main() {
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	float a = texture(u_diffuse, uv).a;
	if (a < 0.1) discard;
}
#endif
"#;

pub struct Material {
	shader: Box<dyn shade::ShaderProgram>,
	shadow_shader: Box<dyn shade::ShaderProgram>,
	texture: Box<dyn shade::Texture2D>,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &*self.texture);
	}
}

pub struct Instance {
	model: Transform3f,
}
impl shade::UniformVisitor for Instance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

struct LightTransformUniforms {
	light_transform: Mat4f,
}
impl shade::UniformVisitor for LightTransformUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_lightTransform", &self.light_transform);
	}
}

pub struct Renderable {
	mesh: shade::d3::VertexMesh,
	material: Material,
	instance: Instance,
}
impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		dataview::embed!(VERTICES: [shade::d3::TexturedVertexN] = "../../assets/oldtree/vertices.bin");
		let mesh = shade::d3::VertexMesh::new(g, Vec3f::ZERO, &VERTICES, shade::BufferUsage::Static);

		let mut source = shade::shader_interface! {
			files {
				"common.glsl" => COMMON,
				"main.glsl" => PROGRAM,
				"shadow.glsl" => SHADOW_PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let shadow_shader = g.shader_compile(&mut source, "shadow.glsl", &[]);
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("assets/oldtree/texture.png").unwrap();
			let props = shade::TextureProps! {
				mip_levels: 8,
				usage: shade::TextureUsage::TEXTURE,
				filter: shade::TextureFilter::Linear,
				wrap: shade::TextureWrap::Repeat,
			};
			g.image(&props.bind(&image))
		};
		let material = Material { shader, shadow_shader, texture };

		let instance = Instance {
			model: Transform3f::translation(Vec3f(0.0, -20.0, 0.0)) * Transform3f::scaling(Vec3::dup(5.0)),
		};

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut shade::Graphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light<'_>, shadow: bool) {
		let light_transform = light.light_view_proj * self.instance.model;
		let uniforms = LightTransformUniforms { light_transform };
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::ALL },
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { &*self.material.shadow_shader } else { &*self.material.shader },
			vertices: &[shade::DrawVertexBuffer {
				buffer: &*self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, light, &self.material, &self.instance, &uniforms],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}

impl super::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {
	}
	fn draw(&self, g: &mut shade::Graphics, globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light<'_>, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, self.instance.model)
	}
}
