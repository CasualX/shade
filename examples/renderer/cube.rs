use std::mem;
use cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	pos: Vec3f,
	normal: Vec3f,
	uv: Vec2f,
	color: [shade::Norm<u8>; 4],
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("a_pos", dataview::offset_of!(Vertex.pos)),
			shade::VertexAttribute::with::<Vec3f>("a_normal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<Vec2f>("a_uv", dataview::offset_of!(Vertex.uv)),
			shade::VertexAttribute::with::<[shade::Norm<u8>; 4]>("a_color", dataview::offset_of!(Vertex.color)),
		],
	};
}

const X_MIN: f32 = -1.0;
const X_MAX: f32 =  1.0;
const Y_MIN: f32 = -1.0;
const Y_MAX: f32 =  1.0;
const Z_MIN: f32 = -1.0;
const Z_MAX: f32 =  1.0;

static VERTICES: [Vertex; 24] = [
	// Front face
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MAX), normal: Vec3(0.0, 0.0, 1.0), uv: Vec2(0.0, 0.0), color: shade::norm!([255,   0,   0, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MAX), normal: Vec3(0.0, 0.0, 1.0), uv: Vec2(1.0, 0.0), color: shade::norm!([192,   0,   0, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MAX), normal: Vec3(0.0, 0.0, 1.0), uv: Vec2(0.0, 1.0), color: shade::norm!([192,   0,   0, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MAX), normal: Vec3(0.0, 0.0, 1.0), uv: Vec2(1.0, 1.0), color: shade::norm!([128,   0,   0, 255]) },
	// Back face
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MIN), normal: Vec3(0.0, 0.0, -1.0), uv: Vec2(0.0, 0.0), color: shade::norm!([  0, 255, 255, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MIN), normal: Vec3(0.0, 0.0, -1.0), uv: Vec2(1.0, 0.0), color: shade::norm!([  0, 192, 192, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MIN), normal: Vec3(0.0, 0.0, -1.0), uv: Vec2(0.0, 1.0), color: shade::norm!([  0, 192, 192, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MIN), normal: Vec3(0.0, 0.0, -1.0), uv: Vec2(1.0, 1.0), color: shade::norm!([  0, 128, 128, 255]) },
	// Left face
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MIN), normal: Vec3(-1.0, 0.0, 0.0), uv: Vec2(0.0, 0.0), color: shade::norm!([  0, 255,   0, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MAX), normal: Vec3(-1.0, 0.0, 0.0), uv: Vec2(1.0, 0.0), color: shade::norm!([  0, 192,   0, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MIN), normal: Vec3(-1.0, 0.0, 0.0), uv: Vec2(0.0, 1.0), color: shade::norm!([  0, 192,   0, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MAX), normal: Vec3(-1.0, 0.0, 0.0), uv: Vec2(1.0, 1.0), color: shade::norm!([  0, 128,   0, 255]) },
	// Right face
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MAX), normal: Vec3(1.0, 0.0, 0.0), uv: Vec2(0.0, 0.0), color: shade::norm!([255,   0, 255, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MIN), normal: Vec3(1.0, 0.0, 0.0), uv: Vec2(1.0, 0.0), color: shade::norm!([192,   0, 192, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MAX), normal: Vec3(1.0, 0.0, 0.0), uv: Vec2(0.0, 1.0), color: shade::norm!([192,   0, 192, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MIN), normal: Vec3(1.0, 0.0, 0.0), uv: Vec2(1.0, 1.0), color: shade::norm!([128,   0, 128, 255]) },
	// Top face
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MAX), normal: Vec3(0.0, 1.0, 0.0), uv: Vec2(0.0, 0.0), color: shade::norm!([  0,   0, 255, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MAX), normal: Vec3(0.0, 1.0, 0.0), uv: Vec2(1.0, 0.0), color: shade::norm!([  0,   0, 192, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MAX, Z_MIN), normal: Vec3(0.0, 1.0, 0.0), uv: Vec2(0.0, 1.0), color: shade::norm!([  0,   0, 192, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MAX, Z_MIN), normal: Vec3(0.0, 1.0, 0.0), uv: Vec2(1.0, 1.0), color: shade::norm!([  0,   0, 128, 255]) },
	// Bottom face
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MIN), normal: Vec3(0.0, -1.0, 0.0), uv: Vec2(0.0, 0.0), color: shade::norm!([255, 255, 255, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MIN), normal: Vec3(0.0, -1.0, 0.0), uv: Vec2(1.0, 0.0), color: shade::norm!([192, 192, 192, 255]) },
	Vertex { pos: Vec3(X_MAX, Y_MIN, Z_MAX), normal: Vec3(0.0, -1.0, 0.0), uv: Vec2(0.0, 1.0), color: shade::norm!([192, 192, 192, 255]) },
	Vertex { pos: Vec3(X_MIN, Y_MIN, Z_MAX), normal: Vec3(0.0, -1.0, 0.0), uv: Vec2(1.0, 1.0), color: shade::norm!([128, 128, 128, 255]) },
];

static INDICES: [u8; 36] = [
	 0, 1, 2,  2, 1, 3, // front
	 4, 5, 6,  6, 5, 7, // back
	 8, 9,10, 10, 9,11, // left
	12,13,14, 14,13,15, // right
	16,17,18, 18,17,19, // top
	20,21,22, 22,21,23, // bottom
];


const CUBE_FS: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec4 v_color;
in vec2 v_uv;
in vec3 v_fragPos;
in vec3 v_normal;
in vec4 v_lightClip;

uniform vec3 u_lightPos;
uniform sampler2D u_shadowMap;
uniform float u_shadowTexelScale;

uniform sampler2D u_texture;

float shadow_visibility(vec4 lightClip, vec3 fragPos) {
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
	vec4 albedo = texture(u_texture, v_uv) * v_color;

	vec3 n = normalize(v_normal);
	n = gl_FrontFacing ? n : -n;
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diff = max(dot(n, lightDir), 0.0);

	float ambient = 0.2;
	float direct_intensity = 0.6;
	float visibility = shadow_visibility(v_lightClip, v_fragPos);
	float lighting = ambient + visibility * (diff * direct_intensity);

	o_fragColor = albedo * lighting;
}
"#;

const CUBE_SHADOW_FS: &str = r#"\
#version 330 core
void main() {
}
"#;

const CUBE_VS: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;
in vec2 a_uv;
in vec4 a_color;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_fragPos;
out vec3 v_normal;
out vec4 v_lightClip;

uniform mat4x3 u_model;
uniform mat4 u_transform;
uniform mat4 u_lightTransform;

void main()
{
	v_color = a_color + vec4(0.5, 0.5, 0.5, 0.0);
	v_uv = a_uv;
	v_fragPos = (u_model * vec4(a_pos, 1.0)).xyz;
	v_normal = (u_model * vec4(a_normal, 0.0)).xyz;
	v_lightClip = u_lightTransform * vec4(a_pos, 1.0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
"#;

//----------------------------------------------------------------
// Cube renderable

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
	model: Transform3f,
}
impl shade::UniformVisitor for Instance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

pub struct Renderable {
	mesh: shade::d3::VertexIndexedMesh,
	material: Material,
	instance: Instance,
}

impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		// Create the vertex and index buffers
		let vertices = g.vertex_buffer(None, &VERTICES, shade::BufferUsage::Static);
		let vertices_len: u32 = VERTICES.len() as u32;
		let indices = g.index_buffer(None, &INDICES, VERTICES.len() as u8, shade::BufferUsage::Static);
		let indices_len = INDICES.len() as u32;

		let mesh = shade::d3::VertexIndexedMesh {
			origin: Vec3f::ZERO,
			bounds: Bounds3f(Vec3f(X_MIN, Y_MIN, Z_MIN), Vec3f(X_MAX, Y_MAX, Z_MAX)),
			vertices,
			vertices_len,
			indices,
			indices_len,
		};

		// Load the texture
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/brick 24 - 256x256.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 8,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			};
			g.image(Some("brick 24"), &(&image, &props))
		};

		// Create the shader
		let shader = g.shader_create(None, CUBE_VS, CUBE_FS);
		let shadow_shader = g.shader_create(None, CUBE_VS, CUBE_SHADOW_FS);

		let material = Material { shader, shadow_shader, texture };

		let instance = Instance { model: Transform3f::IDENTITY };

		Renderable { mesh, material, instance }
	}
	pub fn draw(&self, g: &mut shade::Graphics, _globals: &super::Globals, camera: &shade::d3::Camera, light: &super::Light, shadow: bool) {
		let transform = camera.view_proj * self.instance.model;
		let light_transform = light.light_view_proj * self.instance.model;

		// Draw the cube
		g.draw_indexed(&shade::DrawIndexedArgs {
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
			indices: self.mesh.indices,
			uniforms: &[
				camera,
				light,
				&self.material,
				&shade::UniformFn(|set| {
					set.value("u_model", &self.instance.model);
					set.value("u_transform", &transform);
					set.value("u_lightTransform", &light_transform);
				}),
			],
			index_start: 0,
			index_end: self.mesh.indices_len,
			instances: -1,
		});
	}
}

impl super::IRenderable for Renderable {
	fn update(&mut self, globals: &crate::Globals) {
		let local = Transform3f::translate(Vec3f(-15.0, 0.0, 10.0)) * Transform3f::scale(Vec3::dup(5.0));
		let rot_a = Transform3f::rotate(Vec3f(0.0, 1.0, 0.0), Angle(globals.time * 0.7));
		let rot_b = Transform3f::rotate(Vec3f(1.0, 0.25, 0.1).norm(), Angle(globals.time * 1.3));
		let model = local * rot_a * rot_b;
		self.instance.model = model;
	}
	fn draw(&self, g: &mut shade::Graphics, globals: &crate::Globals, camera: &shade::d3::Camera, light: &crate::Light, shadow: bool) {
		self.draw(g, globals, camera, light, shadow)
	}
	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, self.instance.model)
	}
}
