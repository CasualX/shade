use cvmath::*;

const COMMON: &str = r#"
#ifdef VERTEX_SHADER
in vec3 a_pos;
#endif

VARYING vec3 v_worldPos;

uniform mat4x3 u_viewMatrix;
uniform mat4 u_projMatrix;
uniform vec3 u_globePosition;
uniform float u_globeRadius;

#ifdef VERTEX_SHADER
void main() {
	vec3 world = u_globePosition + a_pos * (1.27 * u_globeRadius);
	vec4 worldPos = vec4(world, 1.0);
	v_worldPos = worldPos.xyz;
	gl_Position = u_projMatrix * mat4(u_viewMatrix) * worldPos;
}
#endif
"#;

const RAYCAST: &str = r#"
vec3 intersectGlobe(vec3 rayOrigin, vec3 rayDir) {
	vec3 oc = rayOrigin - u_globePosition;
	float a = dot(rayDir, rayDir);
	float b = 2.0 * dot(oc, rayDir);
	float c = dot(oc, oc) - u_globeRadius * u_globeRadius;
	float discriminant = b*b - 4.0*a*c;
	if (discriminant < 0.0) {
		return vec3(0.0 / 0.0);
	}
	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) {
		return vec3(0.0 / 0.0);
	}
	return rayOrigin + t * rayDir;
}
"#;

const PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
uniform vec3 u_cameraPosition;
uniform sampler2D u_texture;
uniform vec3 u_lightPos;
uniform sampler2DShadow u_shadowMap;
uniform mat4 u_lightTransform;

const float PI = 3.141592653589793;

#include "raycast.glsl"

void main() {
	vec3 rayDir = normalize(v_worldPos - u_cameraPosition);
	vec3 hitPos = intersectGlobe(u_cameraPosition, rayDir);
	if (any(isnan(hitPos))) {
		discard;
		return;
	}
	vec3 n = normalize(hitPos - u_globePosition);
	vec3 lightDir = normalize(u_lightPos - hitPos);
	float diffLight = max(dot(n, lightDir), 0.0);
	vec4 lightClip = u_lightTransform * vec4(hitPos, 1.0);
	vec3 lightNdc = lightClip.xyz / lightClip.w;
	vec3 shadowUvZ = lightNdc * 0.5 + 0.5;
	float bias = 0.001;
	float visibility = texture(u_shadowMap, vec3(shadowUvZ.xy, shadowUvZ.z - bias));
	float u = 0.5 + atan(n.y, n.x) / (2.0 * PI);
	float v = 0.5 + asin(n.z) / PI;
	v = 1.0 - v;
	u = fract(u);
	vec2 uv = vec2(u, v);
	vec2 uv_dx = dFdx(uv);
	vec2 uv_dy = dFdy(uv);
	if (abs(uv_dx.x) > 0.5) uv_dx.x -= sign(uv_dx.x);
	if (abs(uv_dy.x) > 0.5) uv_dy.x -= sign(uv_dy.x);
	vec3 color = textureGrad(u_texture, uv, uv_dx, uv_dy).rgb;
	float ambient = 0.2;
	float direct_intensity = 0.6;
	float lighting = ambient + visibility * (diffLight * direct_intensity);
	o_fragColor = vec4(color * lighting, 1.0);
}
#endif
"#;

const SHADOW_PROGRAM: &str = r#"
#version unified 330 core
#include "common.glsl"

#ifdef FRAGMENT_SHADER
uniform vec3 u_cameraPosition;

#include "raycast.glsl"

void main() {
	vec3 rayDir = normalize(v_worldPos - u_cameraPosition);
	vec3 hitPos = intersectGlobe(u_cameraPosition, rayDir);
	if (any(isnan(hitPos))) {
		discard;
		return;
	}
	vec4 clip = u_projMatrix * mat4(u_viewMatrix) * vec4(hitPos, 1.0);
	float ndcDepth = clip.z / clip.w;
	gl_FragDepth = ndcDepth * 0.5 + 0.5;
}
#endif
"#;

//----------------------------------------------------------------
// Globe renderable

pub struct Material {
	shader: Box<dyn shade::ShaderProgram>,
	shadow_shader: Box<dyn shade::ShaderProgram>,
	texture: Box<dyn shade::Texture2D>,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_texture", &*self.texture);
	}
}

pub struct Instance {
	position: Vec3f,
	radius: f32,
}
impl shade::UniformVisitor for Instance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_globePosition", &self.position);
		set.value("u_globeRadius", &self.radius);
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
	pub mesh: shade::d3::VertexMesh,
	pub material: Material,
	pub instance: Instance,
}
impl Renderable {
	pub fn create(g: &mut dyn shade::IGraphics) -> Renderable {
		let mesh = shade::d3::icosahedron::icosahedron_flat(g);

		let mut source = shade::shader_interface! {
			files {
				"common.glsl" => COMMON,
				"raycast.glsl" => RAYCAST,
				"main.glsl" => PROGRAM,
				"shadow.glsl" => SHADOW_PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let shadow_shader = g.shader_compile(&mut source, "shadow.glsl", &[]);
		let texture = {
			let image = shade::image::DecodedImage::load_file("assets/textures/2k_earth_daymap.jpg").unwrap();
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
			position: Vec3f(20.0, 20.0, 20.0),
			radius: 10.0,
		};

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut dyn shade::IGraphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light<'_>, shadow: bool) {
		let uniforms = LightTransformUniforms {
			light_transform: light.light_view_proj,
		};
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::LessEqual),
			cull_mode: Some(shade::CullMode::CW),
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::ALL },
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { &*self.material.shadow_shader } else { &*self.material.shader },
			uniforms: &[camera, light, &self.material, &self.instance, &uniforms],
			vertices: &[shade::DrawVertexBuffer {
				buffer: &*self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}

impl super::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {
	}
	fn draw(&self, g: &mut dyn shade::IGraphics, globals: &crate::Globals, camera: &shade::d3::Camera, light: &crate::Light<'_>, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, Transform3f::translation(self.instance.position) * Transform3f::scaling(Vec3f::dup(self.instance.radius)))
	}
}
