use crate::*;

const PROGRAM: &str = r#"
#version unified 330 core, 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec3 a_pos;
#endif

VARYING vec3 v_worldPos;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

uniform mat4x3 u_viewMatrix;
uniform mat4 u_projMatrix;
uniform vec3 u_globePosition;
uniform float u_globeRadius;
uniform vec3 u_cameraPosition;
uniform sampler2D u_texture;

const float PI = 3.141592653589793;

#ifdef VERTEX_SHADER
void main() {
	vec3 world = u_globePosition + a_pos * (1.27 * u_globeRadius);
	vec4 worldPos = vec4(world, 1.0);
	v_worldPos = worldPos.xyz;
	gl_Position = u_projMatrix * vec4(u_viewMatrix * worldPos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	vec3 rayDir = normalize(v_worldPos - u_cameraPosition);
	vec3 rayOrigin = u_cameraPosition;
	vec3 oc = rayOrigin - u_globePosition;
	float a = dot(rayDir, rayDir);
	float b = 2.0 * dot(oc, rayDir);
	float c = dot(oc, oc) - u_globeRadius * u_globeRadius;
	float discriminant = b*b - 4.0*a*c;
	if (discriminant < 0.0) {
		discard;
	}
	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) discard;

	vec3 hitPos = rayOrigin + t * rayDir;
	vec3 n = normalize(hitPos - u_globePosition);
	float u = 0.5 + atan(n.y, n.x) / (2.0 * PI);
	float v = 0.5 + asin(n.z) / PI;
	v = 1.0 - v;
	u = fract(u);

	vec3 color = texture(u_texture, vec2(u, v)).rgb;
	o_fragColor = vec4(color, 1.0);
}
#endif
"#;

struct GlobeMaterial {
	shader: Box<dyn shade::ShaderProgram>,
	texture: Box<dyn shade::Texture2D>,
}

impl shade::UniformVisitor for GlobeMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_texture", &*self.texture);
	}
}

struct GlobeInstance {
	position: Vec3f,
	radius: f32,
}

impl shade::UniformVisitor for GlobeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_globePosition", &self.position);
		set.value("u_globeRadius", &self.radius);
	}
}

struct GlobeRenderable {
	mesh: shade::d3::VertexMesh,
	instance: GlobeInstance,
	material: GlobeMaterial,
}

impl GlobeRenderable {
	fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> GlobeRenderable {
		let mesh = shade::d3::icosahedron::icosahedron_flat(g);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		let texture = {
			let bytes = assets.read("textures/2k_earth_daymap.jpg").unwrap();
			let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
			let props = shade::TextureProps! {
				usage: shade::TextureUsage::TEXTURE,
				filter: shade::TextureFilter::Linear,
				wrap: shade::TextureWrap::Repeat,
			};
			g.image(&props.bind(&image))
		};
		let material = GlobeMaterial { shader, texture };
		let instance = GlobeInstance { position: Vec3f::ZERO, radius: 0.8 };
		GlobeRenderable { mesh, instance, material }
	}

	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::Camera) {
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::LessEqual),
			cull_mode: Some(shade::CullMode::CW),
			mask: shade::DrawMask::ALL,
			prim_type: shade::PrimType::Triangles,
			shader: &*self.material.shader,
			uniforms: &[camera, &self.material, &self.instance],
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

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Globe::new(g, assets))
}

struct Globe {
	screen_size: Vec2i,
	camera: shade::d3::ArcballCamera,
	globe: GlobeRenderable,
	auto_rotate: bool,
	cursor: Vec2f,
	left_click: bool,
	right_click: bool,
	middle_click: bool,
}

impl Globe {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Globe {
		let globe = GlobeRenderable::create(g, assets);
		let camera = shade::d3::ArcballCamera::new(Vec3(0.0, 3.2, 1.8), Vec3::ZERO, Vec3f::Z);
		Globe {
			screen_size: Vec2(1, 1),
			camera,
			globe,
			auto_rotate: true,
			cursor: Vec2f::ZERO,
			left_click: false,
			right_click: false,
			middle_click: false,
		}
	}

	fn camera(&self, viewport: Bounds2i) -> shade::d3::Camera {
		let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
		let position = self.camera.position();
		let hand = Hand::RH;
		let view = self.camera.view_matrix(hand);
		let clip = Clip::NO;
		let (near, far) = (0.1, 100.0);
		let projection = Mat4::perspective(Angle::deg(45.0), aspect_ratio, near, far, (hand, clip));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		shade::d3::Camera { viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip }
	}
}

impl DemoInterface for Globe {
	fn resize(&mut self, size: Vec2i) {
		self.screen_size = Vec2(size.x, size.y);
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseButton { button: gui::MouseButton::LEFT, pressed, .. } => self.left_click = pressed,
			Input::MouseButton { button: gui::MouseButton::MIDDLE, pressed, .. } => self.middle_click = pressed,
			Input::MouseButton { button: gui::MouseButton::RIGHT, pressed, .. } => self.right_click = pressed,
			Input::MouseMove { position } => {
				let delta = position - self.cursor;
				self.cursor = position;
				if self.left_click {
					self.auto_rotate = false;
					self.camera.rotate(-delta.x, -delta.y);
					shell.request_redraw();
				}
				if self.right_click {
					self.auto_rotate = false;
					self.camera.pan_ref(-delta.x, delta.y, Vec3f::Z);
					shell.request_redraw();
				}
				if self.middle_click {
					self.auto_rotate = false;
					self.camera.zoom(delta.y * 0.01);
					shell.request_redraw();
				}
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.05, 0.05, 0.1, 1.0), depth: 1.0);
		if self.auto_rotate {
			let speed = frame.dt * 100.0;
			self.camera.rotate(-speed, 0.0);
		}
		let camera = self.camera(frame.viewport);
		self.globe.draw(g, &camera);
		g.end();
	}
}
