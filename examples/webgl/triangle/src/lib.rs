use std::mem;

mod api;

//----------------------------------------------------------------
// The triangle's vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: cvmath::Vec2f,
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
precision mediump float;

// Varying from vertex shader
varying vec4 VertexColor;

void main()
{
	// Gamma correction (linear -> sRGB)
	gl_FragColor = vec4(
		pow(VertexColor.r, 1.0 / 2.2),
		pow(VertexColor.g, 1.0 / 2.2),
		pow(VertexColor.b, 1.0 / 2.2),
		VertexColor.a
	);
}
"#;

const VERTEX_SHADER: &str = r#"
// Attributes
attribute vec2 aPos;
attribute vec4 aColor;

// Varying to pass to fragment shader
varying vec4 VertexColor;

void main()
{
	// Gamma correction (sRGB -> linear)
	VertexColor = vec4(
		pow(aColor.r, 2.2),
		pow(aColor.g, 2.2),
		pow(aColor.b, 2.2),
		aColor.a
	);

	gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

//----------------------------------------------------------------

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: cvmath::Vec2<i32>,
	shader: shade::Shader,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();

		let g = shade::Graphics(&mut webgl);

		// Create the triangle shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

		let screen_size = cvmath::Vec2::ZERO;
		Context { webgl, screen_size, shader }
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = cvmath::Vec2(width, height);
	}

	pub fn draw(&mut self, time: f64) {
		let g = shade::Graphics(&mut self.webgl);
		g.begin().unwrap();

		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.5, 0.2, 1.0, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Compute rotation matrix from time
		let rotation = cvmath::Mat2::rotate(cvmath::Rad(time as f32));

		// Create the triangle vertices
		let vertices = g.vertex_buffer(None, &[
			TriangleVertex { position: rotation * cvmath::Vec2( 0.0,  0.5), color: [255, 0, 0, 255] },
			TriangleVertex { position: rotation * cvmath::Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
			TriangleVertex { position: rotation * cvmath::Vec2( 0.5, -0.5), color: [0, 0, 255, 255] },
		], shade::BufferUsage::Static).unwrap();

		// Draw the triangle
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::c(0, 0, self.screen_size.x, self.screen_size.y),
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
		}).unwrap();

		g.vertex_buffer_free(vertices, shade::FreeMode::Delete).unwrap();

		g.end().unwrap();
	}
}
