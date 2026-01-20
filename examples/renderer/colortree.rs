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
uniform sampler2DShadow u_shadowMap;

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diffLight = max(dot(v_normal, lightDir), 0.0);

	// Simple shadow mapping
	vec3 lightNdc = v_lightClip.xyz / v_lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float bias = 0.001;
	float visibility = texture(u_shadowMap, vec3(shadowUvZ.xy, shadowUvZ.z - bias));

	// Shadow should only reduce the direct (sun-facing) term.
	// Back-facing surfaces (diffLight ~= 0) then look the same as shadowed ones.
	float ambient = 0.2;
	float direct_intensity = 0.6;
	float lighting = ambient + visibility * (diffLight * direct_intensity);
	o_fragColor = vec4(1.0, 1.0, 1.0, 1.0) * lighting * v_color;
}
"#;

const SHADOW_FRAGMENT_SHADER: &str = r#"\
#version 330 core
void main() {}
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

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_normal = a_normal;
	v_color = srgbToLinear(a_color);
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

pub struct Material {
	shader: shade::Shader,
	shadow_shader: shade::Shader,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, _set: &mut dyn shade::UniformSetter) {
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

pub struct Renderable {
	mesh: shade::d3::VertexMesh,
	material: Material,
	instance: Instance,
}
impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		dataview::embed!(VERTICES: [Vertex] = "../colortree/vertices.bin");
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

impl super::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {
	}
	fn draw(&self, g: &mut shade::Graphics, globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, self.instance.model)
	}
}
