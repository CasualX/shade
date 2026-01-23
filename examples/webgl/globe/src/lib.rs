use shade::cvmath::*;

mod api;

//----------------------------------------------------------------
// Uniforms and shaders

const SPHERE_FS: &str = r#"
precision highp float;

uniform vec3 u_cameraPosition;
uniform vec3 u_globePosition;
uniform float u_globeRadius;
uniform sampler2D u_texture;

varying vec3 v_worldPos;

const float PI = 3.141592653589793;

void main() {
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
		// No intersection - discard fragment
		discard;
	}
	// Nearest positive intersection (handles camera-inside-sphere too)
	float sqrtD = sqrt(discriminant);
	float t0 = (-b - sqrtD) / (2.0 * a);
	float t1 = (-b + sqrtD) / (2.0 * a);
	float t = (t0 > 0.0) ? t0 : t1;
	if (t < 0.0) discard;

	vec3 hitPos = rayOrigin + t * rayDir;
	vec3 n = normalize(hitPos - u_globePosition);

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

	vec3 color = texture2D(u_texture, vec2(u, v)).rgb;
	gl_FragColor = vec4(color, 1.0);
}
"#;

const SPHERE_VS: &str = r#"
precision highp float;

attribute vec3 a_pos;

uniform mat4 u_viewMatrix;
uniform mat4 u_projMatrix;

uniform vec3 u_globePosition;
uniform float u_globeRadius;

varying vec3 v_worldPos;

void main() {
	// The mesh is a unit icosahedron in [-1, 1]^3. Scale it to radius (R) and translate.
	vec3 world = u_globePosition + a_pos * (1.27 * u_globeRadius);
	vec4 worldPos = vec4(world, 1.0);
	v_worldPos = worldPos.xyz;
	gl_Position = u_projMatrix * u_viewMatrix * worldPos;
}
"#;

//----------------------------------------------------------------
// Globe renderable

struct GlobeMaterial {
	shader: shade::ShaderProgram,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for GlobeMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_texture", &self.texture);
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
	fn create(g: &mut shade::Graphics) -> GlobeRenderable {
		let mesh = shade::d3::icosahedron::icosahedron_flat(g);

		let shader = g.shader_compile(SPHERE_VS, SPHERE_FS);
		let texture = {
			let file_jpg = include_bytes!("../../../textures/2k_earth_daymap.jpg");
			let image = shade::image::DecodedImage::load_memory_jpeg(file_jpg).unwrap();
			g.image(&image)
		};
		let material = GlobeMaterial { shader, texture };

		let instance = GlobeInstance {
			position: Vec3f::ZERO,
			radius: 0.8,
		};

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
			shader: self.material.shader,
			uniforms: &[camera, &self.material, &self.instance],
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

//----------------------------------------------------------------
// Context

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	camera: shade::d3::ArcballCamera,
	globe: GlobeRenderable,
	auto_rotate: bool,
	left_click: bool,
	right_click: bool,
	middle_click: bool,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();
		let g = webgl.as_graphics();

		let globe = GlobeRenderable::create(g);

		let camera = shade::d3::ArcballCamera::new(Vec3(0.0, 3.2, 1.8), Vec3::ZERO, Vec3f::Z);

		Context {
			webgl,
			screen_size: Vec2::ZERO,
			camera,
			globe,
			auto_rotate: true,
			left_click: false,
			right_click: false,
			middle_click: false,
		}
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn mousemove(&mut self, dx: f32, dy: f32) {
		if self.left_click {
			self.auto_rotate = false;
			self.camera.rotate(-dx, -dy);
		}
		if self.right_click {
			self.auto_rotate = false;
			self.camera.pan(-dx, dy);
		}
		if self.middle_click {
			self.auto_rotate = false;
			self.camera.zoom(dy * 0.01);
		}
	}

	pub fn mousedown(&mut self, button: u32) {
		match button {
			0 => self.left_click = true,
			1 => self.middle_click = true,
			2 => self.right_click = true,
			_ => {}
		}
	}

	pub fn mouseup(&mut self, button: u32) {
		match button {
			0 => self.left_click = false,
			1 => self.middle_click = false,
			2 => self.right_click = false,
			_ => {}
		}
	}

	pub fn draw(&mut self, _time: f64) {
		let g = self.webgl.as_graphics();

		// Render the frame
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.05, 0.05, 0.1, 1.0), depth: 1.0);

		if self.auto_rotate {
			self.camera.rotate(-1.0, 0.0);
		}

		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 100.0);
			let fov_y = Angle::deg(45.0);
			let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (hand, clip));
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, view, near, far, projection, view_proj, inv_view_proj, clip }
		};

		self.globe.draw(g, &camera);

		// Finish the frame
		g.end();
	}
}
