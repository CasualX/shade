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

#[derive(Clone, Debug)]
pub struct Uniforms {
	pub transform: Mat4f,
	pub colormod: Vec4f,
	pub color_add: Vec4f,
}

impl UniformVisitor for Uniforms {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_colormod", &self.colormod);
		set.value("u_color_add", &self.color_add);
	}
}
