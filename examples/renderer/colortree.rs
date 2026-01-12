use std::mem;
use cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec3f,
	normal: Vec3f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<Vec3f>("a_normal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(Vertex.color)),
		],
	};
}

impl shade::TVertex3 for Vertex {
	#[inline]
	fn position(&self) -> cvmath::Vec3<f32> {
		self.position
	}
}

const FRAGMENT_SHADER: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_normal;
in vec4 v_color;
in vec3 v_fragPos;
in vec4 v_lightClip;

uniform vec3 u_lightPos;
uniform sampler2D u_shadowMap;
uniform float u_shadowTexelScale;

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diff = max(dot(v_normal, lightDir), 0.0);

	// Simple shadow mapping (single tap depth compare)
	vec3 lightNdc = v_lightClip.xyz / v_lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float shadow = 0.0;
	if (shadowUvZ.x >= 0.0 && shadowUvZ.x <= 1.0 && shadowUvZ.y >= 0.0 && shadowUvZ.y <= 1.0 && shadowUvZ.z >= 0.0 && shadowUvZ.z <= 1.0) {
		float currentDepth = shadowUvZ.z;
		float bias = 0.001;
		vec2 texelSize = u_shadowTexelScale / vec2(textureSize(u_shadowMap, 0));
		float occlusion = 0.0;
		for (int y = -1; y <= 1; y++) {
			for (int x = -1; x <= 1; x++) {
				vec2 offset = vec2(float(x), float(y)) * texelSize;
				float closestDepth = texture(u_shadowMap, shadowUvZ.xy + offset).r;
				occlusion += ((currentDepth - bias) > closestDepth) ? 1.0 : 0.0;
			}
		}
		shadow = occlusion / 9.0;
	}

	// Shadow should only reduce the direct (sun-facing) term.
	// Back-facing surfaces (diff ~= 0) then look the same as shadowed ones.
	float ambient = 0.2;
	float direct_intensity = 0.6;
	float visibility = 1.0 - shadow;
	float lighting = ambient + visibility * (diff * direct_intensity);
	o_fragColor = vec4(1.0, 1.0, 1.0, 1.0) * lighting * v_color;
}
"#;

const SHADOW_FRAGMENT_SHADER: &str = r#"\
#version 330 core
void main() {
}
"#;

const VERTEX_SHADER: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;
in vec4 a_color;

out vec3 v_normal;
out vec4 v_color;
out vec3 v_fragPos;
out vec4 v_lightClip;

uniform mat4x3 u_model;

uniform mat4 u_transform;
uniform mat4 u_lightTransform;

void main()
{
	v_normal = a_normal;
	v_color = a_color;
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

pub struct Material {
	pub shader: shade::Shader,
	pub shadow_shader: shade::Shader,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, _set: &mut dyn shade::UniformSetter) {
	}
}

pub struct Instance {
	pub model: Transform3f,
}
impl shade::UniformVisitor for Instance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

pub struct Renderable {
	pub mesh: shade::d3::VertexMesh,
	pub material: Material,
	pub instance: Instance,
}
impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		shade::include_bin!(VERTICES: [Vertex] = "../colortree/vertices.bin");
		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &VERTICES, shade::BufferUsage::Static);

		// Create the material
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);
		let shadow_shader = g.shader_create(None, VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);
		let material = Material { shader, shadow_shader };

		let instance = Instance {
			model: Transform3f::scale(Vec3f::dup(0.01)),
		};

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut shade::Graphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		let transform = camera.view_proj * self.instance.model;
		let light_transform = light.light_view_proj * self.instance.model;
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::Less),
			cull_mode: None,
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { self.material.shadow_shader } else { self.material.shader },
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[
				camera,
				light,
				&self.material,
				&self.instance,
				&shade::UniformFn(|set| {
					set.value("u_transform", &transform);
					set.value("u_lightTransform", &light_transform);
				}),
			],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}
