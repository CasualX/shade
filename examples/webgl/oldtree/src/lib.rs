use std::{io, mem, slice};
use shade::cvmath::*;

mod api;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec3f,
	normal: Vec3f,
	uv: Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<Vec3f>("aPos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<Vec3f>("aNormal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<Vec2f>("aUV", dataview::offset_of!(Vertex.uv)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
precision mediump float;

varying vec3 v_normal;
varying vec2 v_uv;
varying vec3 v_fragPos;

uniform sampler2D u_diffuse;
uniform vec3 u_position;

void main() {
	// Define light direction (normalized)
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));

	// Calculate diffuse lighting
	vec3 norm = normalize(v_normal);
	float diff = max(dot(norm, lightDir), 0.0);

	// Sample texture and discard transparent fragments
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	vec4 texColor = texture2D(u_diffuse, uv);
	if (texColor.a < 0.1) {
		discard;
	}

	// Apply quantized diffuse lighting to texture color
	vec3 finalColor = texColor.rgb * (0.4 + diff * 0.8);

	vec3 viewDir = normalize(u_position - v_fragPos);
	float rim = 1.0 - max(dot(viewDir, norm), 0.0);
	rim = smoothstep(0.5, 0.6, rim);
	finalColor += vec3(1.0, 0.8, 0.5) * rim * 0.2;  // warm rim glow

	gl_FragColor = vec4(finalColor, texColor.a);

	// gl_FragColor = vec4(norm * 0.5 + 0.5, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
precision mediump float;

attribute vec3 aPos;
attribute vec3 aNormal;
attribute vec2 aUV;

varying vec3 v_fragPos;
varying vec3 v_normal;
varying vec2 v_uv;

uniform mat4 u_model;
uniform mat4 u_view_proj;

uniform mat3 u_normalMatrix;

void main()
{
	// Calculate world position of the vertex
	v_fragPos = vec3(u_model * vec4(aPos, 1.0));

	// Transform the normal properly (especially for scaling)
	v_normal = u_normalMatrix * aNormal;

	// Pass through UV
	v_uv = aUV;

	// Final position for rasterization
	gl_Position = u_view_proj * vec4(v_fragPos, 1.0);
}
"#;

//----------------------------------------------------------------

struct OldTreeInstance {
	model: Transform3f,
	light_pos: Vec3f,
}

impl shade::UniformVisitor for OldTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		let normal_matrix = self.model.mat3().inverse().transpose();
		set.value("u_normalMatrix", &normal_matrix);
		set.value("lightPos", &self.light_pos);
	}
}

struct OldTreeModel {
	shader: shade::Shader,
	vertices: shade::VertexBuffer,
	vertices_len: u32,
	texture: shade::Texture2D,
	bounds: Bounds3<f32>,
}

impl shade::UniformVisitor for OldTreeModel {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.texture);
	}
}

static VERTICES_DATA: ([u64; 0], [u8; 306816]) = ([], *include_bytes!("../../../oldtree/vertices.bin"));

impl OldTreeModel {
	fn create(g: &mut shade::Graphics) -> OldTreeModel {
		let vertices = unsafe { slice::from_raw_parts(VERTICES_DATA.1.as_ptr() as *const Vertex, VERTICES_DATA.1.len() / mem::size_of::<Vertex>()) };

		let mut mins = Vec3::dup(f32::INFINITY);
		let mut maxs = Vec3::dup(f32::NEG_INFINITY);
		for v in vertices {
			mins = mins.min(v.position);
			maxs = maxs.max(v.position);
			// println!("Vertex {}: {:?}", i, v);
		}
		let bounds = Bounds3(mins, maxs);

		let vertices_len = vertices.len() as u32;
		let vertices = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static);

		let texture = include_bytes!("../../../oldtree/texture.png");

		let texture = shade::image::png::load(g, None, &mut io::Cursor::new(texture), &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		// Create the shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		OldTreeModel { shader, vertices, vertices_len, texture, bounds }
	}

	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &OldTreeInstance) {
		// Draw the model
		g.draw(&shade::DrawArgs {
			surface: camera.surface,
			viewport: camera.viewport,
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
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
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[self, camera, instance],
			vertex_start: 0,
			vertex_end: self.vertices_len,
			instances: -1,
		});
	}
}

//----------------------------------------------------------------

#[allow(dead_code)]
enum ProjectionType {
	Perspective,
	Orthographic,
}
#[allow(dead_code)]
impl ProjectionType {
	fn toggle(&mut self) {
		*self = match *self {
			ProjectionType::Perspective => ProjectionType::Orthographic,
			ProjectionType::Orthographic => ProjectionType::Perspective,
		};
	}
}

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	projection_type: ProjectionType,
	camera: shade::d3::ArcballCamera,
	tree: OldTreeModel,
	auto_rotate: bool,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();

		let ref mut g = shade::Graphics(&mut webgl);

		let tree = OldTreeModel::create(g);

		let camera = {
			let pivot = tree.bounds.center().set_x(0.0).set_y(0.0);
			let position = pivot + Vec3::<f32>::X * tree.bounds.size().xy().vmax();

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

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn draw(&mut self, _time: f64) {
		let g = shade::Graphics(&mut self.webgl);

		// Render the frame
		g.begin();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		if self.auto_rotate {
			self.camera.rotate(-1.0, 0.0);
		}

		let camera = {
			let surface = shade::Surface::BACK_BUFFER;
			let viewport = Bounds2::vec(self.screen_size);
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 40.0);
			let projection = match self.projection_type {
				ProjectionType::Perspective => Mat4::perspective(Angle::deg(90.0), self.screen_size.x as f32 / self.screen_size.y as f32, near, far, (hand, clip)),
				ProjectionType::Orthographic => Mat4::ortho(-5.0 * aspect_ratio, 5.0 * aspect_ratio, -5.0, 5.0, near, far, (hand, clip)),
			};
			let view_proj = projection * view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { surface, viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		self.tree.draw(g, &camera, &OldTreeInstance {
			model: Transform3f::IDENTITY,
			light_pos: Vec3(4.0, 0.0, -230.0),
		});

		// Finish the frame
		g.end();
	}
}
