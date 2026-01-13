use cvmath::*;

const SPHERE_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_worldPos;

uniform vec3 u_cameraPosition;
uniform vec3 u_globePosition;
uniform float u_globeRadius;
uniform sampler2D u_texture;

uniform vec3 u_lightPos;
uniform sampler2D u_shadowMap;
uniform float u_shadowTexelScale;

uniform mat4 u_lightTransform;

const float PI = 3.141592653589793;

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

void main()
{
	// Ray from camera through fragment
	vec3 rayDir = normalize(v_worldPos - u_cameraPosition);
	vec3 rayOrigin = u_cameraPosition;

	// Sphere centered at globePosition (world space)
	vec3 oc = rayOrigin - u_globePosition;

	float a = dot(rayDir, rayDir);
	float b = 2.0 * dot(oc, rayDir);
	float c = dot(oc, oc) - u_globeRadius * u_globeRadius;

	float discriminant = b*b - 4.0*a*c;
	if (discriminant < 0.0) {
	// Purple debug color
		// o_fragColor = vec4(0.5, 0.0, 0.5, 1.0);
		discard;
		return;
	}
	// Nearest positive intersection (handles camera-inside-sphere too)
	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) discard;

	vec3 hitPos = rayOrigin + t * rayDir;
	vec3 n = normalize(hitPos - u_globePosition);

	vec3 lightDir = normalize(u_lightPos - hitPos);
	float diff = max(dot(n, lightDir), 0.0);
	float visibility = shadow_visibility(u_lightTransform * vec4(hitPos, 1.0));

	// Spherical UVs (equirectangular)
	// World is Z-up in this demo (see ArcballCamera::new(..., up = Z)).
	// Longitude around +Z axis, latitude from equator toward +Z.
	float u = 0.5 + atan(n.y, n.x) / (2.0 * PI);
	float v = 0.5 + asin(n.z) / PI;

	// PNG rows decode top-to-bottom; OpenGL UV (0,0) samples the first row.
	// Flip V so the image appears upright.
	v = 1.0 - v;

	// Keep within [0,1) for wrapping samplers.
	u = fract(u);

	vec2 uv = vec2(u, v);
	vec2 uv_dx = dFdx(uv);
	vec2 uv_dy = dFdy(uv);
	// Fix discontinuity at the wrap seam so implicit mip selection doesn't explode.
	if (abs(uv_dx.x) > 0.5) uv_dx.x -= sign(uv_dx.x);
	if (abs(uv_dy.x) > 0.5) uv_dy.x -= sign(uv_dy.x);

	vec3 color = textureGrad(u_texture, uv, uv_dx, uv_dy).rgb;
	float ambient = 0.2;
	float direct_intensity = 0.6;
	float lighting = ambient + visibility * (diff * direct_intensity);
	o_fragColor = vec4(color * lighting, 1.0);
}
"#;

const SPHERE_SHADOW_FS: &str = r#"\
#version 330 core

in vec3 v_worldPos;

uniform vec3 u_cameraPosition;
uniform vec3 u_globePosition;
uniform float u_globeRadius;

uniform mat4x3 u_viewMatrix;
uniform mat4 u_projMatrix;

void main()
{
	// Ray from light-camera through fragment
	vec3 rayDir = normalize(v_worldPos - u_cameraPosition);
	vec3 rayOrigin = u_cameraPosition;
	vec3 oc = rayOrigin - u_globePosition;

	float a = dot(rayDir, rayDir);
	float b = 2.0 * dot(oc, rayDir);
	float c = dot(oc, oc) - u_globeRadius * u_globeRadius;

	float discriminant = b*b - 4.0*a*c;
	if (discriminant < 0.0) {
		discard;
		return;
	}

	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) discard;

	vec3 hitPos = rayOrigin + t * rayDir;
	vec4 clip = u_projMatrix * mat4(u_viewMatrix) * vec4(hitPos, 1.0);
	float ndcDepth = clip.z / clip.w;
	gl_FragDepth = ndcDepth * 0.5 + 0.5;
}
"#;

const SPHERE_VS: &str = r#"\
#version 330 core

in vec3 a_pos;

uniform mat4x3 u_viewMatrix;
uniform mat4 u_projMatrix;

uniform vec3 u_globePosition;
uniform float u_globeRadius;

out vec3 v_worldPos;

void main()
{
	// The mesh is a unit icosahedron in [-1, 1]^3. Scale it to radius (R) and translate.
	vec3 world = u_globePosition + a_pos * (1.27 * u_globeRadius);
	vec4 worldPos = vec4(world, 1.0);
	v_worldPos = worldPos.xyz;
	gl_Position = u_projMatrix * mat4(u_viewMatrix) * worldPos;
}
"#;

//----------------------------------------------------------------
// Globe renderable

pub struct Material {
	shader: shade::Shader,
	shadow_shader: shade::Shader,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_texture", &self.texture);
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

pub struct Renderable {
	pub mesh: shade::d3::VertexMesh,
	pub material: Material,
	pub instance: Instance,
}
impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		let mesh = shade::d3::icosahedron::icosahedron_flat(g);

		let shader = g.shader_create(None, SPHERE_VS, SPHERE_FS);
		let shadow_shader = g.shader_create(None, SPHERE_VS, SPHERE_SHADOW_FS);
		let texture = {
			let image = shade::image::DecodedImage::load_file("examples/textures/2k_earth_daymap.jpg").unwrap();
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
			position: Vec3f(20.0, 20.0, 20.0),
			radius: 10.0,
		};

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut shade::Graphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::LessEqual),
			cull_mode: Some(shade::CullMode::CW),
			mask: if shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::ALL },
			prim_type: shade::PrimType::Triangles,
			shader: if shadow { self.material.shadow_shader } else { self.material.shader },
			uniforms: &[
				camera,
				light,
				&self.material,
				&self.instance,
				&shade::UniformFn(|set| {
					set.value("u_lightTransform", &light.light_view_proj);
				}),
			],
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
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
	fn draw(&self, g: &mut shade::Graphics, globals: &crate::Globals, camera: &shade::d3::Camera, light: &crate::Light, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, Transform3f::translate(self.instance.position) * Transform3f::scale(Vec3f::dup(self.instance.radius)))
	}
}
