use std::collections::HashMap;
use std::{fs, mem};
use cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct BunnyVertex {
	position: Vec3f,
	normal: Vec3f,
}

unsafe impl shade::TVertex for BunnyVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<BunnyVertex>() as u16,
		alignment: mem::align_of::<BunnyVertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(BunnyVertex.position)),
			shade::VertexAttribute::with::<Vec3f>("a_normal", dataview::offset_of!(BunnyVertex.normal)),
		],
	};
}

impl shade::TVertex3 for BunnyVertex {
	#[inline]
	fn position(&self) -> cvmath::Vec3<f32> {
		self.position
	}
}

const BUNNY_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_fragPos;
in vec3 v_normal;
in vec4 v_lightClip;

uniform vec3 u_lightPos;
uniform sampler2D u_shadowMap;
uniform float u_shadowTexelScale;

float shadow_visibility(vec4 lightClip) {
	vec3 lightNdc = lightClip.xyz / lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	if (shadowUvZ.x < 0.0 || shadowUvZ.x > 1.0 || shadowUvZ.y < 0.0 || shadowUvZ.y > 1.0 || shadowUvZ.z < 0.0 || shadowUvZ.z > 1.0) {
		return 1.0;
	}
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
	float shadow = occlusion / 9.0;
	return 1.0 - shadow;
}

void main() {
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diff = max(dot(normalize(v_normal), lightDir), 0.0);

	float ambient = 0.2;
	float direct_intensity = 0.6;
	float visibility = shadow_visibility(v_lightClip);
	float lighting = ambient + visibility * (diff * direct_intensity);

	o_fragColor = vec4(1.0, 1.0, 1.0, 1.0) * lighting;
}
"#;

const BUNNY_SHADOW_FS: &str = r#"\
#version 330 core
void main() {
}
"#;

const BUNNY_VS: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;

out vec3 v_fragPos;
out vec3 v_normal;
out vec4 v_lightClip;

uniform mat4 u_transform;
uniform mat4x3 u_model;
uniform mat4 u_lightTransform;

void main()
{
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

pub struct Material {
	shader: shade::Shader,
	shadow_shader: shade::Shader,
}

pub struct Instance {
	model: Transform3f,
}

pub struct Renderable {
	mesh: shade::d3::VertexMesh,
	material: Material,
	instance: Instance,
}

impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		let mut bunny_file = fs::File::open("examples/models/Bunny-LowPoly.stl").unwrap();
		let bunny_stl = stl::read_stl(&mut bunny_file).unwrap();

		let mut vertices = Vec::new();
		for triangle in bunny_stl.triangles.iter() {
			vertices.push(BunnyVertex {
				position: triangle.v1.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(BunnyVertex {
				position: triangle.v2.into(),
				normal: triangle.normal.into(),
			});
			vertices.push(BunnyVertex {
				position: triangle.v3.into(),
				normal: triangle.normal.into(),
			});
		}

		// Smooth the normals
		let mut map = HashMap::new();
		for v in vertices.iter() {
			map.entry(v.position.map(f32::to_bits)).or_insert(Vec::new()).push(v.normal);
		}
		for v in &mut vertices {
			let normals = map.get(&v.position.map(f32::to_bits)).unwrap();
			let mut normal = Vec3::ZERO;
			for n in normals.iter() {
				normal += *n;
			}
			v.normal = normal.norm();
		};

		// Create the vertex and index buffers
		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &vertices, shade::BufferUsage::Static);

		// println!("Bunny # vertices: {}", mesh.vertices_len);
		// println!("Bunny bounds: {:#?}", mesh.bounds);

		// Create the shader
		let shader = g.shader_create(None, BUNNY_VS, BUNNY_FS);
		let shadow_shader = g.shader_create(None, BUNNY_VS, BUNNY_SHADOW_FS);
		let material = Material { shader, shadow_shader };
		let instance = Instance {
			model: Transform3f::translate(Vec3f(-10.0, 30.0, 0.0)) * Transform3f::scale(Vec3::dup(0.25)),
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
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::COLOR | shade::DrawMask::DEPTH },
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { self.material.shadow_shader } else { self.material.shader },
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[
				camera,
				light,
				&shade::UniformFn(|set| {
					set.value("u_transform", &transform);
					set.value("u_model", &self.instance.model);
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
