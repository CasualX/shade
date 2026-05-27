use shade::cvmath::*;

const PROGRAM: &str = r#"#version unified 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec3 a_pos;
in vec3 a_normal;
in vec2 a_uv;
#endif

VARYING vec3 v_fragPos;
VARYING vec3 v_normal;
VARYING vec2 v_uv;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

uniform mat4x3 u_model;
uniform mat4 u_viewProjMatrix;
uniform mat3 u_normalMatrix;
uniform sampler2D u_diffuse;
uniform vec3 u_cameraPosition;

#ifdef VERTEX_SHADER
void main() {
	v_fragPos = vec3(u_model * vec4(a_pos, 1.0));
	v_normal = u_normalMatrix * a_normal;
	v_uv = a_uv;
	gl_Position = u_viewProjMatrix * vec4(v_fragPos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));
	vec3 norm = normalize(v_normal);
	float diff = max(dot(norm, lightDir), 0.0);
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	vec4 texColor = texture(u_diffuse, uv);
	if (texColor.a < 0.1) {
		discard;
	}
	vec3 finalColor = texColor.rgb * (0.4 + diff * 0.8);
	o_fragColor = vec4(finalColor, texColor.a);
}
#endif
"#;

struct OldTreeMaterial {
	shader: shade::ShaderProgram,
	texture: shade::Texture2D,
}

impl shade::UniformVisitor for OldTreeMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.texture);
	}
}

struct OldTreeInstance {
	model: Transform3f,
}

impl shade::UniformVisitor for OldTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		let normal_matrix = self.model.mat3().inverse().transpose();
		set.value("u_normalMatrix", &normal_matrix);
	}
}

struct OldTreeRenderable {
	mesh: shade::d3::VertexMesh,
	material: OldTreeMaterial,
	instance: OldTreeInstance,
}

impl OldTreeRenderable {
	fn create(g: &mut shade::Graphics) -> OldTreeRenderable {
		dataview::embed!(VERTICES: [shade::d3::TexturedVertexN] = "../../oldtree/vertices.bin");
		let mesh = shade::d3::VertexMesh::new(g, Vec3f::ZERO, &VERTICES, shade::BufferUsage::Static);

		let texture = {
			let file_png = include_bytes!("../../oldtree/texture.png");
			let image = shade::image::DecodedImage::load_memory_png(file_png).unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Nearest,
				filter_mag: shade::TextureFilter::Nearest,
				wrap_u: shade::TextureWrap::Edge,
				wrap_v: shade::TextureWrap::Edge,
				..Default::default()
			};
			g.image(&(&image, &props))
		};

		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let material = OldTreeMaterial { shader, texture };
		let instance = OldTreeInstance { model: Transform3f::IDENTITY };

		OldTreeRenderable { mesh, material, instance }
	}

	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::Camera, light: &Light) {
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: shade::DrawMask {
				red: true,
				green: true,
				blue: false,
				alpha: true,
				depth: true,
				stencil: 0,
			},
			prim_type: shade::PrimType::Triangles,
			shader: self.material.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, &self.material, &self.instance, light],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}

struct Light {
	light_pos: Vec3f,
}

impl shade::UniformVisitor for Light {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_lightPos", &self.light_pos);
	}
}

#[allow(dead_code)]
enum ProjectionType {
	Perspective,
	Orthographic,
}

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	projection_type: ProjectionType,
	camera: shade::d3::ArcballCamera,
	tree: OldTreeRenderable,
	auto_rotate: bool,
}

impl Context {
	pub fn new() -> Context {
		shade::webgl::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		});
		let g = webgl.as_graphics();
		let tree = OldTreeRenderable::create(g);

		let camera = {
			let pivot = tree.mesh.bounds.center().set_x(0.0).set_y(0.0);
			let position = pivot + Vec3::<f32>::X * tree.mesh.bounds.size().xy().vmax();
			shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
		};

		Context {
			webgl,
			screen_size: Vec2::ZERO,
			projection_type: ProjectionType::Perspective,
			camera,
			tree,
			auto_rotate: true,
		}
	}
}

impl crate::DemoContext for Context {
	fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	fn draw(&mut self, _time: f64) {
		let g = self.webgl.as_graphics();
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		shade::clear!(g, color: Vec4(0.5, 0.2, 0.2, 1.0), depth: 1.0);

		if self.auto_rotate {
			self.camera.rotate(-1.0, 0.0);
		}

		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 40.0);
			let projection = match self.projection_type {
				ProjectionType::Perspective => Mat4::perspective(Angle::deg(90.0), aspect_ratio, near, far, (hand, clip)),
				ProjectionType::Orthographic => Mat4::ortho(-5.0 * aspect_ratio, 5.0 * aspect_ratio, -5.0, 5.0, near, far, (hand, clip)),
			};
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		let light = Light { light_pos: Vec3(4.0, 0.0, -230.0) };
		self.tree.draw(g, &camera, &light);
		g.end();
	}
}
