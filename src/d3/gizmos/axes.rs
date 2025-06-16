use super::*;

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct Vertex {
	pub position: Vec3f,
	pub color: [u8; 4],
}

unsafe impl TVertex for Vertex {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			VertexAttribute { name: "a_pos", format: VertexAttributeFormat::F32v3, offset: dataview::offset_of!(Vertex.position) as u16 },
			VertexAttribute { name: "a_color", format: VertexAttributeFormat::U8Normv4, offset: dataview::offset_of!(Vertex.color) as u16 },
		],
	};
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Uniform {
	pub transform: Mat4f,
}

unsafe impl TUniform for Uniform {
	const LAYOUT: &'static UniformLayout = &UniformLayout {
		size: mem::size_of::<Uniform>() as u16,
		alignment: mem::align_of::<Uniform>() as u16,
		fields: &[
			UniformField { name: "u_transform", ty: UniformType::Mat4x4 { layout: MatrixLayout::RowMajor }, offset: dataview::offset_of!(Uniform.transform) as u16, len: 1 },
		],
	};
}

pub struct Axes {
	pub shader: Shader,
	pub vertices: VertexBuffer,
	pub vertices_len: u32,
	pub indices: IndexBuffer,
	pub indices_len: u32,
}

impl Axes {
	pub fn create(g: &mut Graphics, shader: Shader) -> Axes {
		let vertices = g.vertex_buffer(None, &VERTICES, BufferUsage::Static).unwrap();
		let vertices_len = VERTICES.len() as u32;
		let indices = g.index_buffer(None, &INDICES, BufferUsage::Static).unwrap();
		let indices_len = INDICES.len() as u32;

		Axes { shader, vertices, vertices_len, indices, indices_len }
	}
}

static VERTICES: [Vertex; 27] = [
	// X axis
	Vertex { position: Vec3f::new(0.0, 0.0, 0.0), color: [255, 0, 0, 255] }, // 0
	Vertex { position: Vec3f::new(1.0, 0.0, 0.0), color: [255, 0, 0, 255] }, // 1
	Vertex { position: Vec3f::new(0.9, 0.05, 0.0), color: [255, 0, 0, 255] }, // 2
	Vertex { position: Vec3f::new(1.0, 0.0, 0.0), color: [255, 0, 0, 255] }, // 3
	Vertex { position: Vec3f::new(0.9, -0.05, 0.0), color: [255, 0, 0, 255] }, // 4
	Vertex { position: Vec3f::new(1.1, 0.04, 0.04), color: [255, 0, 0, 255] }, // 5
	Vertex { position: Vec3f::new(1.1, -0.04, -0.04), color: [255, 0, 0, 255] }, // 6
	Vertex { position: Vec3f::new(1.1, 0.04, -0.04), color: [255, 0, 0, 255] }, // 7
	Vertex { position: Vec3f::new(1.1, -0.04, 0.04), color: [255, 0, 0, 255] }, // 8

	// Y axis
	Vertex { position: Vec3f::new(0.0, 0.0, 0.0), color: [0, 255, 0, 255] }, // 9
	Vertex { position: Vec3f::new(0.0, 1.0, 0.0), color: [0, 255, 0, 255] }, // 10
	Vertex { position: Vec3f::new(0.05, 0.9, 0.0), color: [0, 255, 0, 255] }, // 11
	Vertex { position: Vec3f::new(0.0, 1.0, 0.0), color: [0, 255, 0, 255] }, // 12
	Vertex { position: Vec3f::new(-0.05, 0.9, 0.0), color: [0, 255, 0, 255] }, // 13
	Vertex { position: Vec3f::new(0.03, 1.1, 0.04), color: [0, 255, 0, 255] }, // 14
	Vertex { position: Vec3f::new(0.0, 1.1, 0.0), color: [0, 255, 0, 255] }, // 15
	Vertex { position: Vec3f::new(-0.03, 1.1, 0.04), color: [0, 255, 0, 255] }, // 16
	Vertex { position: Vec3f::new(0.0, 1.1, -0.04), color: [0, 255, 0, 255] }, // 17

	// Z axis
	Vertex { position: Vec3f::new(0.0, 0.0, 0.0), color: [0, 64, 255, 255] }, // 18
	Vertex { position: Vec3f::new(0.0, 0.0, 1.0), color: [0, 64, 255, 255] }, // 19
	Vertex { position: Vec3f::new(0.05, 0.0, 0.9), color: [0, 64, 255, 255] }, // 20
	Vertex { position: Vec3f::new(0.0, 0.0, 1.0), color: [0, 64, 255, 255] }, // 21
	Vertex { position: Vec3f::new(-0.05, 0.0, 0.9), color: [0, 64, 255, 255] }, // 22
	Vertex { position: Vec3f::new(-0.04, 0.04, 1.1), color: [0, 64, 255, 255] }, // 23
	Vertex { position: Vec3f::new(0.04, 0.04, 1.1), color: [0, 64, 255, 255] }, // 24
	Vertex { position: Vec3f::new(-0.04, -0.04, 1.1), color: [0, 64, 255, 255] }, // 25
	Vertex { position: Vec3f::new(0.04, -0.04, 1.1), color: [0, 64, 255, 255] }, // 26
];

static INDICES: [u16; 34] = [
	// X axis
	0, 1,       // axis line
	2, 3, 3, 4, // arrowhead lines
	5, 6, 7, 8, // letter X lines

	// Y axis
	9, 10,      // axis line
	11, 12, 12, 13, // arrowhead lines
	14, 15, 16, 15, 15, 17, // letter Y lines

	// Z axis
	18, 19,     // axis line
	20, 21, 21, 22, // arrowhead lines
	23, 24, 24, 25, 25, 26, // letter Z lines
];
