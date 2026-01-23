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

void main() {
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

uniform sampler2DShadow u_shadowMap;

uniform float u_heightScale;

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
vec2 parallaxOcclusionMap(vec2 uv, vec3 viewDirTangent, vec2 uv_dx, vec2 uv_dy) {
	const float minLayers = 8.0;
	const float maxLayers = 48.0;

	// More layers at grazing angles.
	float ndotv = clamp(viewDirTangent.z, 0.0, 1.0);
	float numLayers = mix(maxLayers, minLayers, ndotv);
	float layerDepth = 1.0 / numLayers;

	// Scale parallax by view angle to avoid excessive stepping artifacts.
	vec2 P = (viewDirTangent.xy / max(viewDirTangent.z, 0.05)) * u_heightScale;
	vec2 deltaUV = -P / numLayers;

	vec2 currUV = uv;
	float currLayerDepth = 0.0;
	float currHeight = textureGrad(u_heightMap, currUV, uv_dx, uv_dy).r;

	int steps = int(numLayers);
	for (int i = 0; i < steps; i++) {
		if (currLayerDepth >= currHeight) {
			break;
		}
		currUV += deltaUV;
		currLayerDepth += layerDepth;
		currHeight = textureGrad(u_heightMap, currUV, uv_dx, uv_dy).r;
	}

	// Refine: linearly interpolate between the last two steps to hide layer bands.
	vec2 prevUV = currUV - deltaUV;
	float prevLayerDepth = currLayerDepth - layerDepth;
	float prevHeight = textureGrad(u_heightMap, prevUV, uv_dx, uv_dy).r;

	float after = currHeight - currLayerDepth;
	float before = prevHeight - prevLayerDepth;
	float denom = (before - after);
	float t = (abs(denom) < 1e-5) ? 0.0 : clamp(before / denom, 0.0, 1.0);
	return mix(prevUV, currUV, t);
}

void main() {
	vec2 uv_dx = dFdx(v_uv);
	vec2 uv_dy = dFdy(v_uv);

	// Compute TBN matrix
	mat3 TBN = computeTBN(normalize(v_normal), v_fragPos, v_uv);
	vec3 viewDir = normalize(u_cameraPosition - v_fragPos);
	vec3 viewDirTangent = TBN * viewDir;

	// Perform Parallax Occlusion Mapping
	vec2 displacedUV = parallaxOcclusionMap(v_uv, viewDirTangent, uv_dx, uv_dy);

	// Optional: Clamp UVs to avoid artifacts at the edges
	// if (displacedUV.x < 0.0 || displacedUV.x > 1.0 || displacedUV.y < 0.0 || displacedUV.y > 1.0)
	// 	discard;

	// Sample diffuse texture
	vec4 texColor = textureGrad(u_diffuse, displacedUV, uv_dx, uv_dy);
	if (texColor.a < 0.1)
		discard;

	// Sample and decode the normal map (assumed in [0,1] range)
	vec3 normalTangent = textureGrad(u_normalMap, displacedUV, uv_dx, uv_dy).rgb * 2.0 - 1.0;

	// Transform to world space
	vec3 perturbedNormal = normalize(TBN * normalTangent);

	// Lighting
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diffLight = max(dot(perturbedNormal, lightDir), 0.0);

	// Simple shadow mapping
	vec3 lightNdc = v_lightClip.xyz / v_lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float bias = 0.001;
	float visibility = texture(u_shadowMap, vec3(shadowUvZ.xy, shadowUvZ.z - bias));

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
	shader: shade::ShaderProgram,
	shadow_shader: shade::ShaderProgram,
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

		let mesh = shade::d3::VertexMesh::new(g, Vec3f::ZERO, &vertices, shade::BufferUsage::Static);

		let props = shade::TextureProps {
			mip_levels: 8,
			usage: shade::TextureUsage::TEXTURE,
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Repeat,
			wrap_v: shade::TextureWrap::Repeat,
			..Default::default()
		};

		let diffuse = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.diffuse.png").unwrap();
			g.image(&(&image, &props))
		};

		let normal_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.normal.png").unwrap();
			g.image(&(&image, &props))
		};

		let height_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.height.png").unwrap();
			g.image(&(&image, &props))
		};

		let shader = g.shader_compile(VERTEX_SHADER, PARALLAX_SHADER);
		let shadow_shader = g.shader_compile(VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);
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
			depth_test: Some(shade::Compare::Less),
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
