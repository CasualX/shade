use std::mem;
use shade::cvmath::*;

mod api;

//----------------------------------------------------------------
// The triangle's vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: Vec2f,
	color: [u8; 4],
}

unsafe impl shade::TVertex for TriangleVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<TriangleVertex>() as u16,
		alignment: mem::align_of::<TriangleVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "aPos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TriangleVertex.position) as u16,
			},
			shade::VertexAttribute {
				name: "aColor",
				format: shade::VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TriangleVertex.color) as u16,
			},
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
precision highp float;

// Varying from vertex shader
varying vec4 VertexColor;

void main() {
	gl_FragColor = VertexColor;
}
"#;

const VERTEX_SHADER: &str = r#"
// Attributes
attribute vec2 aPos;
attribute vec4 aColor;

// Varying to pass to fragment shader
varying vec4 VertexColor;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	// Gamma correction (sRGB -> linear)
	VertexColor = srgbToLinear(aColor);

	gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

//----------------------------------------------------------------

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	shader: shade::Shader,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();
		let g = webgl.as_graphics();

		// Create the triangle shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

		let screen_size = Vec2::ZERO;
		Context { webgl, screen_size, shader }
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn draw(&mut self, time: f64) {
		let g = self.webgl.as_graphics();
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		shade::clear!(g, color: Vec4(0.2, 0.5, 0.2, 1.0), depth: 1.0);

		// Compute rotation matrix from time
		let rotation = Mat2::rotate(Angle(time as f32));

		// Create the triangle vertices
		let vertices = g.vertex_buffer(None, &[
			TriangleVertex { position: rotation * Vec2( 0.0,  0.5), color: [255, 0, 0, 255] },
			TriangleVertex { position: rotation * Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
			TriangleVertex { position: rotation * Vec2( 0.5, -0.5), color: [0, 0, 255, 255] },
		], shade::BufferUsage::Static);

		// Draw the triangle
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::ALL,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[],
			vertex_start: 0,
			vertex_end: 3,
			instances: -1,
		});

		g.vertex_buffer_free(vertices, shade::FreeMode::Delete);

		g.end();
	}
}
