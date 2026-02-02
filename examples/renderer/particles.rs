use cvmath::*;

const N: i32 = 64;

const PARTICLE_VS: &str = r#"
#version 330 core
in vec2 a_pos;
in vec2 a_uv;
in vec4 a_color;

uniform sampler2D u_positions;
uniform mat4 u_viewProjMatrix;
uniform mat4x3 u_viewMatrix;

out vec2 v_uv;

void main() {
	ivec2 particle_index = ivec2(gl_InstanceID % 64, gl_InstanceID / 64);
	vec3 particle_pos = texelFetch(u_positions, particle_index, 0).rgb;

	vec3 cam_right = vec3(u_viewMatrix[0][0], u_viewMatrix[1][0], u_viewMatrix[2][0]);
	vec3 cam_up = vec3(u_viewMatrix[0][1], u_viewMatrix[1][1], u_viewMatrix[2][1]);
	vec3 world_pos = particle_pos + cam_right * a_pos.x + cam_up * a_pos.y;
	gl_Position = u_viewProjMatrix * vec4(world_pos, 1.0);
	v_uv = a_uv;
}
"#;

const PARTICLE_FS: &str = r#"
#version 330 core
in vec2 v_uv;
uniform sampler2D u_texture;
out vec4 frag_color;

void main() {
	frag_color = texture(u_texture, v_uv) * 0.5;
	if (frag_color.a < 0.1) {
		discard;
	}
}
"#;

const UPDATE_POSITIONS_FS: &str = r#"
#version 330 core

in vec2 a_pos;
in vec2 a_uv;

uniform sampler2D u_positions;
uniform float u_deltaTime;

uniform vec3 u_boundsMins;
uniform vec3 u_boundsMaxs;
uniform float u_time;

out vec3 out_position;

void main() {
	vec3 position = texelFetch(u_positions, ivec2(gl_FragCoord.xy), 0).rgb;
	vec3 velocity = vec3(-4.0, 2.5, -10.0);
	vec3 wind = vec3(
		sin(u_time * 0.9 + position.y * 0.05),
		cos(u_time * 0.9 + position.x * 0.05),
		sin(u_time * 0.9 + (position.x + position.y) * 0.05)
	) * 20.0;
	velocity += wind * 0.5;
	position += velocity * u_deltaTime;

	// Wrap around bounds
	for (int i = 0; i < 3; i++) {
		if (position[i] < u_boundsMins[i]) {
			position[i] = u_boundsMaxs[i];
		}
		if (position[i] > u_boundsMaxs[i]) {
			position[i] = u_boundsMins[i];
		}
	}

	out_position = position;
}
"#;

struct Material {
	shader: shade::ShaderProgram,
	texture: shade::Texture2D,
	positions: shade::Texture2D,
}
impl shade::UniformVisitor for Material {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_positions", &self.positions);
		set.value("u_texture", &self.texture);
	}
}

struct Instance {
	bounds: Bounds3f,
}
impl shade::UniformVisitor for Instance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_boundsMins", &self.bounds.mins);
		set.value("u_boundsMaxs", &self.bounds.maxs);
	}
}

pub struct Renderable {
	mesh: shade::VertexBuffer,
	material: Material,
	instance: Instance,

	positions: [shade::Texture2D; 2],
	update_shader: shade::ShaderProgram,
	index: bool,
	pp: shade::d2::PostProcessQuad,
}

static INSTANCE_QUAD: [shade::d2::TexturedVertex; 6] = [
	shade::d2::TexturedVertex { pos: Vec2f(-0.5, -0.5), uv: Vec2f(0.0, 0.0), color: Vec4::dup(255) },
	shade::d2::TexturedVertex { pos: Vec2f(0.5, -0.5), uv: Vec2f(1.0, 0.0), color: Vec4::dup(255) },
	shade::d2::TexturedVertex { pos: Vec2f(0.5, 0.5), uv: Vec2f(1.0, 1.0), color: Vec4::dup(255) },
	shade::d2::TexturedVertex { pos: Vec2f(-0.5, -0.5), uv: Vec2f(0.0, 0.0), color: Vec4::dup(255) },
	shade::d2::TexturedVertex { pos: Vec2f(0.5, 0.5), uv: Vec2f(1.0, 1.0), color: Vec4::dup(255) },
	shade::d2::TexturedVertex { pos: Vec2f(-0.5, 0.5), uv: Vec2f(0.0, 1.0), color: Vec4::dup(255) },
];

impl Renderable {
	pub fn create(g: &mut shade::Graphics) -> Renderable {
		let bounds = Bounds3f::new(
			Vec3f(-64.0, -64.0, 0.0),
			Vec3f(64.0, 64.0, 64.0),
		);
		let instance = Instance { bounds };

		let texture = shade::image::DecodedImage::load_file("examples/textures/snowflake.png").unwrap();
		let texture = g.image(&(
			&texture,
			&shade::TextureProps {
				mip_levels: 8,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Edge,
				wrap_v: shade::TextureWrap::Edge,
				..Default::default()
			},
		));

		let pos_info = shade::Texture2DInfo {
			width: N,
			height: N,
			format: shade::TextureFormat::RGB32F,
			props: shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage!(WRITE | SAMPLED | COLOR_TARGET),
				filter_min: shade::TextureFilter::Nearest,
				filter_mag: shade::TextureFilter::Nearest,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
				..Default::default()
			},
		};
		let positions = g.texture2d_create(&pos_info);
		let mut initial_data = Vec::with_capacity((N * N * 3) as usize);
		let mut rng = urandom::new();
		for _y in 0..N {
			for _x in 0..N {
				let px = rng.range(bounds.mins.x..bounds.maxs.x);
				let py = rng.range(bounds.mins.y..bounds.maxs.y);
				let pz = rng.range(bounds.mins.z..bounds.maxs.z);
				initial_data.push(px);
				initial_data.push(py);
				initial_data.push(pz);
			}
		}
		g.texture2d_write(positions, 0, dataview::bytes(initial_data.as_slice()));

		let positions2 = g.texture2d_create(&pos_info);

		let shader = g.shader_compile(PARTICLE_VS, PARTICLE_FS).unwrap();
		let material = Material { shader, texture, positions };

		let mesh = g.vertex_buffer(&INSTANCE_QUAD, shade::BufferUsage::Static);

		let pp = shade::d2::PostProcessQuad::create(g);
		let update_shader = g.shader_compile(shade::gl::shaders::POST_PROCESS_VS, UPDATE_POSITIONS_FS).unwrap();

		Renderable { mesh, material, instance, positions: [positions, positions2], index: false, pp, update_shader }
	}

	pub fn update_positions(&mut self, g: &mut shade::Graphics, globals: &crate::Globals) {
		let src = if self.index { 1 } else { 0 };
		let dst = if self.index { 0 } else { 1 };
		self.index = !self.index;

		let src = self.positions[src];
		let dst = self.positions[dst];

		self.material.positions = dst;

		g.begin(&shade::BeginArgs::Immediate {
			viewport: Bounds2i::new(Vec2i::ZERO, Vec2i(N, N)),
			color: &[dst],
			levels: None,
			depth: shade::Texture2D::INVALID,
		});

		self.pp.draw(g, self.update_shader, shade::BlendMode::Solid, &[
			globals,
			&shade::UniformFn(|set| {
				set.value("u_positions", &src);
				set.value("u_deltaTime", &0.01f32);
				set.value("u_boundsMins", &self.instance.bounds.mins);
				set.value("u_boundsMaxs", &self.instance.bounds.maxs);
			}),
		]);

		g.end();
	}
}

impl crate::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {}

	fn draw(&self, g: &mut shade::Graphics, globals: &crate::Globals, camera: &shade::d3::Camera, light: &crate::Light, _shadow: bool) {
		// if _shadow {
		// 	false;
		// }

		let mask = if _shadow { shade::DrawMask::DEPTH } else { shade::DrawMask::COLOR };

		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Additive,
			depth_test: Some(shade::Compare::LessEqual),
			cull_mode: None,
			mask,
			prim_type: shade::PrimType::Triangles,
			shader: self.material.shader,
			uniforms: &[
				camera,
				light,
				globals,
				&self.material,
				&self.instance,
			],
			vertices: &[
				shade::DrawVertexBuffer {
					buffer: self.mesh,
					divisor: shade::VertexDivisor::PerVertex,
				},
			],
			vertex_start: 0,
			vertex_end: INSTANCE_QUAD.len() as u32,
			instances: N * N,
		})
	}
	fn get_bounds(&self) -> (cvmath::Bounds3f, cvmath::Transform3f) {
		(self.instance.bounds, Transform3f::IDENTITY)
	}
}
