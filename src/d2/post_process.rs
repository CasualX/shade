use super::*;

/// A full-screen quad for post-processing effects.
pub struct PostProcessQuad {
	vertices: VertexBuffer,
}

impl PostProcessQuad {
	/// Creates a new instance.
	pub fn create(g: &mut Graphics) -> PostProcessQuad {
		let vertices = g.vertex_buffer(&VERTICES, BufferUsage::Static);
		PostProcessQuad { vertices }
	}

	/// Draw the post-process quad with the given shader and blend mode.
	///
	/// # Shader
	///
	/// The shader should expect the following vertex attributes:
	/// - `a_pos`: `vec2` - The position of the vertex in normalized device coordinates (NDC).
	/// - `a_uv`: `vec2` - The texture coordinates of the vertex.
	///
	/// # Blend Mode
	///
	/// The blend mode determines how the quad's output is blended with the existing framebuffer content.
	///
	/// # Uniforms
	///
	/// The `uniforms` parameter allows you to pass additional uniform data to the shader.
	/// This can include textures, colors, or any other data your shader requires.
	pub fn draw(&self, g: &mut Graphics, shader: ShaderProgram, blend_mode: BlendMode, uniforms: &[&dyn UniformVisitor]) {
		g.draw(&DrawArgs {
			scissor: None,
			blend_mode,
			depth_test: None,
			cull_mode: None,
			mask: DrawMask::COLOR,
			prim_type: PrimType::Triangles,
			shader,
			uniforms,
			vertices: &[DrawVertexBuffer {
				buffer: self.vertices,
				divisor: VertexDivisor::PerVertex,
			}],
			vertex_start: 0,
			vertex_end: 6,
			instances: -1,
		});
	}
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
struct Vertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
}

unsafe impl dataview::Pod for Vertex {}

unsafe impl TVertex for Vertex {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "a_pos",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.pos) as u16,
			},
			VertexAttribute {
				name: "a_uv",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.uv) as u16,
			},
		],
	};
}

static VERTICES: [Vertex; 6] = [
	Vertex { pos: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
	Vertex { pos: Vec2f( 1.0, -1.0), uv: Vec2f(1.0, 0.0) },
	Vertex { pos: Vec2f( 1.0,  1.0), uv: Vec2f(1.0, 1.0) },
	Vertex { pos: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
	Vertex { pos: Vec2f( 1.0,  1.0), uv: Vec2f(1.0, 1.0) },
	Vertex { pos: Vec2f(-1.0,  1.0), uv: Vec2f(0.0, 1.0) },
];
