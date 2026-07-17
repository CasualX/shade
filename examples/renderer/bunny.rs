use cvmath::*;

const COMMON: &str = r#"
#ifdef VERTEX_SHADER
in vec3 a_pos;
in vec3 a_normal;
#endif

VARYING vec3 v_fragPos;
VARYING vec3 v_normal;
VARYING vec4 v_lightClip;

uniform mat4 u_transform;
uniform mat4x3 u_model;
uniform mat4 u_lightTransform;

#ifdef VERTEX_SHADER
void main() {
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
#endif
"#;

const PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
uniform vec3 u_lightPos;
uniform sampler2DShadow u_shadowMap;

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diffLight = max(dot(normalize(v_normal), lightDir), 0.0);
	vec3 lightNdc = v_lightClip.xyz / v_lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float bias = 0.001;
	float visibility = texture(u_shadowMap, vec3(shadowUvZ.xy, shadowUvZ.z - bias));
	float ambient = 0.2;
	float direct_intensity = 0.6;
	float lighting = ambient + visibility * (diffLight * direct_intensity);
	o_fragColor = vec4(1.0, 1.0, 1.0, 1.0) * lighting;
}
#endif
"#;

const SHADOW_PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
void main() {}
#endif
"#;

pub struct Material {
	shader: Box<dyn shade::ShaderProgram>,
	shadow_shader: Box<dyn shade::ShaderProgram>,
}

pub struct Instance {
	model: Transform3f,
}

struct TransformUniforms {
	transform: Mat4f,
	model: Transform3f,
	light_transform: Mat4f,
}
impl shade::UniformVisitor for TransformUniforms {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_model", &self.model);
		set.value("u_lightTransform", &self.light_transform);
	}
}

#[allow(dead_code)]
pub struct Renderable {
	bounds: Bounds3f,
	vertices: Box<dyn shade::VertexBuffer>,
	vertices_len: u32,
	indices_buffer: Box<dyn shade::IndexBuffer>,
	indices_len: u32,
	material: Material,
	instance: Instance,
}

impl Renderable {
	pub fn create(g: &mut dyn shade::IGraphics) -> Renderable {
		let bunny = shade::model::stl::StlFile::load_file("assets/models/Bunny-LowPoly.stl").unwrap();
		let bunny = bunny.smooth_vertices(None);

		// Create the vertex and index buffers
		let bounds = bunny.bounds;
		let vertices = g.vertex_buffer(&bunny.vertices, shade::BufferUsage::Static);
		let vertices_len = bunny.vertices.len() as u32;
		let indices_buffer = g.index_buffer(&bunny.indices, vertices_len, shade::BufferUsage::Static);
		let indices_len = bunny.indices.len() as u32 * 3;

		// Create the shader
		let mut source = shade::shader_interface! {
			files {
				"common.glsl" => COMMON,
				"main.glsl" => PROGRAM,
				"shadow.glsl" => SHADOW_PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let shadow_shader = g.shader_compile(&mut source, "shadow.glsl", &[]);
		let material = Material { shader, shadow_shader };
		let instance = Instance {
			model: Transform3f::translation(Vec3f(-10.0, 30.0, 0.0)) * Transform3f::scaling(Vec3::dup(0.25)),
		};

		Renderable { bounds, vertices, vertices_len, indices_buffer, indices_len, material, instance }
	}
	pub fn draw(&self, g: &mut dyn shade::IGraphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		let transform = camera.view_proj * self.instance.model;
		let light_transform = light.light_view_proj * self.instance.model;
		let uniforms = TransformUniforms {
			transform,
			model: self.instance.model,
			light_transform,
		};

		g.draw_indexed(&shade::DrawIndexedArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::Less),
			cull_mode: None,
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::COLOR | shade::DrawMask::DEPTH },
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { &*self.material.shadow_shader } else { &*self.material.shader },
			vertices: &[
				shade::DrawVertexBuffer {
					buffer: &*self.vertices,
					divisor: shade::VertexDivisor::PerVertex,
				},
			],
			uniforms: &[camera, light, &uniforms],
			indices: &*self.indices_buffer,
			index_start: 0,
			index_end: self.indices_len,
			instances: -1,
		});
	}
}

impl super::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {
	}
	fn draw(&self, g: &mut dyn shade::IGraphics, globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light<'_>, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.bounds, self.instance.model)
	}
}
