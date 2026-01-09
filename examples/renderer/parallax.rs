use cvmath::*;

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


const PARALLAX_SHADER: &str = r#"
#version 330 core

out vec4 o_fragColor;

in vec2 v_uv;
in vec3 v_normal;
in vec3 v_fragPos;
in vec4 v_lightClip;

uniform sampler2D u_diffuse;
uniform sampler2D u_normalMap;
uniform sampler2D u_heightMap;
uniform vec3 u_cameraPosition;
uniform vec3 u_lightPos;

uniform sampler2D u_shadowMap;
uniform float u_shadowTexelScale;

uniform float u_heightScale;

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

// Construct TBN matrix using derivatives (quad assumed)
mat3 computeTBN(vec3 normal, vec3 pos, vec2 uv) {
	vec3 dp1 = dFdx(pos);
	vec3 dp2 = dFdy(pos);
	vec2 duv1 = dFdx(uv);
	vec2 duv2 = dFdy(uv);

	vec3 t = normalize(dp1 * duv2.y - dp2 * duv1.y);
	vec3 b = normalize(cross(normal, t));
	return mat3(t, -b, normal);
}

// Parallax Occlusion Mapping
vec2 parallaxOcclusionMap(vec2 uv, vec3 viewDirTangent) {
	const int numLayers = 32;
	const float minLayers = 8.0;
	const float maxLayers = 32.0;

	float angle = dot(vec3(0.0, 0.0, 1.0), viewDirTangent);
	float num = mix(maxLayers, minLayers, abs(angle));
	float layerDepth = 1.0 / num;
	vec2 P = viewDirTangent.xy * u_heightScale;
	vec2 deltaUV = -P / num;

	vec2 currUV = uv;
	float currDepth = 0.0;
	float heightFromMap = texture(u_heightMap, currUV).r;

	// Step until depth of map is below current layer
	while (currDepth < heightFromMap && num > 0.0) {
		currUV += deltaUV;
		currDepth += layerDepth;
		heightFromMap = texture(u_heightMap, currUV).r;
	}

	return currUV;
}

void main() {
	// Compute TBN matrix
	mat3 TBN = computeTBN(normalize(v_normal), v_fragPos, v_uv);
	vec3 viewDir = normalize(u_cameraPosition - v_fragPos);
	vec3 viewDirTangent = TBN * viewDir;

	// Perform Parallax Occlusion Mapping
	vec2 displacedUV = parallaxOcclusionMap(v_uv, viewDirTangent);

	// Optional: Clamp UVs to avoid artifacts at the edges
	// if (displacedUV.x < 0.0 || displacedUV.x > 1.0 || displacedUV.y < 0.0 || displacedUV.y > 1.0)
	// 	discard;

	// Sample diffuse texture
	vec4 texColor = texture(u_diffuse, displacedUV);
	if (texColor.a < 0.1)
		discard;

	// Sample and decode the normal map (assumed in [0,1] range)
	vec3 normalTangent = texture(u_normalMap, displacedUV).rgb * 2.0 - 1.0;

	// Transform to world space
	vec3 perturbedNormal = normalize(TBN * normalTangent);

	// Lighting
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diffLight = max(dot(perturbedNormal, lightDir), 0.0);
	float visibility = shadow_visibility(v_lightClip);

	// Final color
	vec3 finalColor = texColor.rgb * (0.6 + visibility * (diffLight * 0.4));
	o_fragColor = vec4(finalColor, texColor.a);
}
"#;

const SHADOW_FRAGMENT_SHADER: &str = r#"\
#version 330 core

in vec2 v_uv;
uniform sampler2D u_diffuse;

void main() {
	float a = texture(u_diffuse, v_uv).a;
	if (a < 0.1) discard;
}
"#;

pub struct Material {
	shader: shade::Shader,
	shadow_shader: shade::Shader,
	diffuse: shade::Texture2D,
	normal_map: shade::Texture2D,
	height_map: shade::Texture2D,
	height_scale: f32,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.diffuse);
		set.value("u_normalMap", &self.normal_map);
		set.value("u_heightMap", &self.height_map);
		set.value("u_heightScale", &self.height_scale);
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
		let vertices = [
			shade::d3::TexturedVertexN { pos: Vec3f(-5.0, -5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(0.0, 2.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(5.0, -5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(2.0, 2.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(5.0, 5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(2.0, 0.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(-5.0, 5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(0.0, 0.0) },
		];
		let indices = [0, 1, 2, 0, 2, 3];
		let vertices = indices.map(|i| vertices[i]);

		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &vertices, shade::BufferUsage::Static);

		let diffuse = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.diffuse.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				border_color: [0, 0, 0, 0],
			};
			g.image(None, &(&image, &props))
		};

		let normal_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.normal.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				border_color: [0, 0, 0, 0],
			};
			g.image(None, &(&image, &props))
		};

		let height_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.height.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				border_color: [0, 0, 0, 0],
			};
			g.image(None, &(&image, &props))
		};

		let shader = g.shader_create(None, VERTEX_SHADER, PARALLAX_SHADER);
		let shadow_shader = g.shader_create(None, VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);
		let material = Material {
			shader,
			shadow_shader,
			diffuse,
			normal_map,
			height_map,
			height_scale: 0.04,
		};

		let instance = Instance {
			model: Transform3f::scale(Vec3::dup(10.0)),
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
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::ALL },
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
