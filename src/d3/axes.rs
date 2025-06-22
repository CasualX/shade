use super::*;

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct AxesVertex {
	pub position: Vec3f,
	pub color: [u8; 4],
}

unsafe impl TVertex for AxesVertex {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<AxesVertex>() as u16,
		alignment: mem::align_of::<AxesVertex>() as u16,
		attributes: &[
			VertexAttribute { name: "a_pos", format: VertexAttributeFormat::F32v3, offset: dataview::offset_of!(AxesVertex.position) as u16 },
			VertexAttribute { name: "a_color", format: VertexAttributeFormat::U8Normv4, offset: dataview::offset_of!(AxesVertex.color) as u16 },
		],
	};
}

#[derive(Clone, Debug)]
pub struct AxesInstance {
	pub local: Mat4f,
}

impl UniformVisitor for AxesInstance {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_local", &self.local);
	}
}

impl UniformVisitor for (&camera::CameraSetup, &AxesInstance) {
	fn visit(&self, set: &mut dyn UniformSetter) {
		let (camera, instance) = self;
		camera.visit(set);
		instance.visit(set);
		let transform = camera.view_proj * instance.local;
		set.value("u_transform", &transform);
	}
}

pub struct AxesModel {
	pub shader: Shader,
	pub vertices: VertexBuffer,
	pub vertices_len: u32,
	pub indices: IndexBuffer,
	pub indices_len: u32,
}

impl AxesModel {
	pub fn create(g: &mut Graphics, shader: Shader) -> AxesModel {
		let vertices = g.vertex_buffer(None, &VERTICES, BufferUsage::Static).unwrap();
		let vertices_len = VERTICES.len() as u32;
		let indices = g.index_buffer(None, &INDICES, BufferUsage::Static).unwrap();
		let indices_len = INDICES.len() as u32;

		AxesModel { shader, vertices, vertices_len, indices, indices_len }
	}

	pub fn draw(&self, g: &mut Graphics, camera: &camera::CameraSetup, instance: &AxesInstance) {
		g.draw_indexed(&DrawIndexedArgs {
			surface: Surface::BACK_BUFFER,
			viewport: camera.viewport,
			scissor: None,
			blend_mode: BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: DrawMask::ALL,
			prim_type: PrimType::Lines,
			shader: self.shader,
			vertices: &[DrawVertexBuffer {
				buffer: self.vertices,
				divisor: VertexDivisor::PerVertex,
			}],
			uniforms: &[&(camera, instance)],
			indices: self.indices,
			index_start: 0,
			vertex_start: 0,
			vertex_end: self.vertices_len,
			index_end: self.indices_len,
			instances: -1,
		}).unwrap();
	}
}

const RED: [u8; 4] = [245, 59, 39, 255];
const GREEN: [u8; 4] = [181, 255, 104, 255];
const BLUE: [u8; 4] = [7, 111, 255, 255];

static VERTICES: [AxesVertex; 27] = [
	// X axis [0]
	AxesVertex { position: Vec3f::new(0.0, 0.0, 0.0), color: RED },
	AxesVertex { position: Vec3f::new(1.0, 0.0, 0.0), color: RED },
	AxesVertex { position: Vec3f::new(0.9, 0.05, 0.0), color: RED },
	AxesVertex { position: Vec3f::new(1.0, 0.0, 0.0), color: RED },
	AxesVertex { position: Vec3f::new(0.9, -0.05, 0.0), color: RED },
	AxesVertex { position: Vec3f::new(1.1, 0.04, 0.04), color: RED },
	AxesVertex { position: Vec3f::new(1.1, -0.04, -0.04), color: RED },
	AxesVertex { position: Vec3f::new(1.1, 0.04, -0.04), color: RED },
	AxesVertex { position: Vec3f::new(1.1, -0.04, 0.04), color: RED },

	// Y axis [9]
	AxesVertex { position: Vec3f::new(0.0, 0.0, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(0.0, 1.0, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(0.05, 0.9, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(0.0, 1.0, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(-0.05, 0.9, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(0.03, 1.1, 0.04), color: GREEN },
	AxesVertex { position: Vec3f::new(0.0, 1.1, 0.0), color: GREEN },
	AxesVertex { position: Vec3f::new(-0.03, 1.1, 0.04), color: GREEN },
	AxesVertex { position: Vec3f::new(0.0, 1.1, -0.04), color: GREEN },

	// Z axis [18]
	AxesVertex { position: Vec3f::new(0.0, 0.0, 0.0), color: BLUE },
	AxesVertex { position: Vec3f::new(0.0, 0.0, 1.0), color: BLUE },
	AxesVertex { position: Vec3f::new(0.05, 0.0, 0.9), color: BLUE },
	AxesVertex { position: Vec3f::new(0.0, 0.0, 1.0), color: BLUE },
	AxesVertex { position: Vec3f::new(-0.05, 0.0, 0.9), color: BLUE },
	AxesVertex { position: Vec3f::new(-0.04, 0.04, 1.1), color: BLUE },
	AxesVertex { position: Vec3f::new(0.04, 0.04, 1.1), color: BLUE },
	AxesVertex { position: Vec3f::new(-0.04, -0.04, 1.1), color: BLUE },
	AxesVertex { position: Vec3f::new(0.04, -0.04, 1.1), color: BLUE },
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
