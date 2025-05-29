mod api;

//----------------------------------------------------------------
// The triangle's vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: cvmath::Vec2<f32>,
	color: [u8; 4],
}

unsafe impl shade::TVertex for TriangleVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: std::mem::size_of::<TriangleVertex>() as u16,
		alignment: std::mem::align_of::<TriangleVertex>() as u16,
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
// No version directive
precision mediump float; // Must declare default precision in fragment shader

// Varying from vertex shader
varying vec4 VertexColor;

void main()
{
	// Gamma correction (linear -> sRGB), component-wise
	gl_FragColor = vec4(
		pow(VertexColor.r, 1.0 / 2.2),
		pow(VertexColor.g, 1.0 / 2.2),
		pow(VertexColor.b, 1.0 / 2.2),
		VertexColor.a
	);
}
"#;

const VERTEX_SHADER: &str = r#"
// No version directive

// Attributes
attribute vec2 aPos;
attribute vec4 aColor;

// Varying to pass to fragment shader
varying vec4 VertexColor;

void main()
{
	// Gamma correction (sRGB -> linear), component-wise
	VertexColor = vec4(
		pow(aColor.r, 2.2),
		pow(aColor.g, 2.2),
		pow(aColor.b, 2.2),
		aColor.a
	);

	// Set gl_Position (z = 0.0, w = 1.0)
	gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

//----------------------------------------------------------------

extern "C" {
	fn consoleLog(ptr: *const u8, len: usize);
}

fn log(s: &str) {
	unsafe { consoleLog(s.as_ptr(), s.len()) };
}

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	shader: shade::Shader,
	vb: shade::Buffer,
}

impl Context {
	pub fn new() -> Context {
		let mut webgl = shade::webgl::WebGLGraphics::new();

		let g = shade::Graphics(&mut webgl);
		// Create the triangle vertex buffer
		let vb = g.buffer(None, &[
			TriangleVertex { position: cvmath::Vec2( 0.0,  0.5), color: [255, 0, 0, 255] },
			TriangleVertex { position: cvmath::Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
			TriangleVertex { position: cvmath::Vec2( 0.5, -0.5), color: [0, 0, 255, 255] },
		], shade::BufferUsage::Static).unwrap();

		log(&format!("Vertex buffer created {:?}", vb));


		// Create the triangle shader
		let shader = g.shader_create(None).unwrap();
		if let Err(_) = g.shader_compile(shader, VERTEX_SHADER, FRAGMENT_SHADER) {
			panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
		}


		Context { webgl, shader, vb }
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

		// Draw the triangle
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Rect::c(0, 0, 1532, 1005),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vb,
				divisor: shade::VertexDivisor::PerVertex,
				layout: <TriangleVertex as shade::TVertex>::LAYOUT,
			}],
			uniforms: shade::UniformRef::default(),
			vertex_start: 0,
			vertex_end: 3,
			instances: -1,
		}).unwrap();

		g.end().unwrap();
	}
}
