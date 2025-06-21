use super::*;

/// Color vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct ColorVertex {
	pub pos: Point2f,
	pub color: Vec4<u8>,
}

unsafe impl TVertex for ColorVertex {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<ColorVertex>() as u16,
		alignment: mem::align_of::<ColorVertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "a_pos",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(ColorVertex.pos) as u16,
			},
			VertexAttribute {
				name: "a_color",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(ColorVertex.color) as u16,
			},
		],
	};
}

/// Color template.
#[derive(Copy, Clone, Debug)]
pub struct ColorTemplate {
	pub color: Vec4<u8>,
}

impl ToVertex<ColorVertex> for ColorTemplate {
	#[inline]
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> ColorVertex {
		ColorVertex { pos, color: self.color }
	}
}

/// Color uniform.
#[derive(Clone, Debug)]
pub struct ColorUniform {
	pub transform: Transform2f,
	pub pattern: Transform2f,
	pub colormod: Vec4f,
}

impl UniformVisitor for ColorUniform {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_pattern", &self.pattern);
		set.value("u_colormod", &self.colormod);
	}
}
