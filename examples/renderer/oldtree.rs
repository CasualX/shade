use cvmath::*;

const FRAGMENT_SHADER: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_normal;
in vec2 v_uv;
in vec3 v_fragPos;
in vec4 v_lightClip;

uniform sampler2D u_diffuse;
uniform vec3 u_cameraPosition;
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
	// Define light direction (normalized)
	vec3 lightDir = normalize(u_lightPos - v_fragPos);

	// Calculate diffuse lighting
	vec3 norm = normalize(v_normal);
	float diff = max(dot(norm, lightDir), 0.0);

	// Sample texture and discard transparent fragments
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	vec4 texColor = texture(u_diffuse, uv);
	if (texColor.a < 0.1) {
		discard;
	}

	float visibility = shadow_visibility(v_lightClip);
	// Shadow should only reduce the direct (sun-facing) term.
	vec3 finalColor = texColor.rgb * (0.4 + visibility * (diff * 0.8));

	vec3 viewDir = normalize(u_cameraPosition - v_fragPos);
	float rim = 1.0 - max(dot(viewDir, norm), 0.0);
	rim = smoothstep(0.5, 0.6, rim);
	finalColor += vec3(1.0, 0.8, 0.5) * rim * 0.2;  // warm rim glow

	o_fragColor = vec4(finalColor, texColor.a);

	// o_fragColor = vec4(norm * 0.5 + 0.5, 1.0);
}
"#;

const SHADOW_FRAGMENT_SHADER: &str = r#"\
#version 330 core

in vec2 v_uv;
uniform sampler2D u_diffuse;

void main() {
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	float a = texture(u_diffuse, uv).a;
	if (a < 0.1) discard;
}
"#;

const VERTEX_SHADER: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;
in vec2 a_uv;

out vec3 v_fragPos;
out vec3 v_normal;
out vec2 v_uv;
out vec4 v_lightClip;

uniform mat4x3 u_model;

uniform mat4 u_viewProjMatrix;
uniform mat4 u_lightTransform;

void main()
{
	// Calculate world position of the vertex
	v_fragPos = vec3(u_model * vec4(a_pos, 1.0));

	// Transform the normal properly (especially for scaling)
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;

	// Pass through UV
	v_uv = a_uv;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);

	// Final position for rasterization
	gl_Position = u_viewProjMatrix * vec4(v_fragPos, 1.0);
}
"#;

pub struct Material {
	shader: shade::Shader,
	shadow_shader: shade::Shader,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.texture);
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
		shade::include_bin!(VERTICES: [shade::d3::TexturedVertexN] = "../oldtree/vertices.bin");
		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &VERTICES, shade::BufferUsage::Static);

		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);
		let shadow_shader = g.shader_create(None, VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/oldtree/texture.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 8,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			};
			g.image(None, &(&image, &props))
		};
		let material = Material { shader, shadow_shader, texture };

		let instance = Instance {
			model: Transform3f::translate(Vec3f(0.0, -20.0, 0.0)) * Transform3f::scale(Vec3::dup(5.0)),
		};

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut shade::Graphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		let light_transform = light.light_view_proj * self.instance.model;
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: if shadow {
				shade::DrawMask::DEPTH
			} else {
				shade::DrawMask::ALL
			},
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
					set.value("u_lightTransform", &light_transform);
				}),
			],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}
