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
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
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
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ColorUniform {
	pub transform: Transform2f,
	pub pattern: Transform2f,
	pub colormod: Vec4f,
}

impl Default for ColorUniform {
	fn default() -> Self {
		ColorUniform {
			transform: Transform2f::IDENTITY,
			pattern: Transform2f::IDENTITY,
			colormod: Vec4::dup(1.0f32),
		}
	}
}

unsafe impl TUniform for ColorUniform {
	const LAYOUT: &'static UniformLayout = &UniformLayout {
		size: mem::size_of::<ColorUniform>() as u16,
		alignment: mem::align_of::<ColorUniform>() as u16,
		fields: &[
			UniformField {
				name: "u_transform",
				ty: UniformType::Mat3x2 { layout: MatrixLayout::RowMajor },
				offset: dataview::offset_of!(ColorUniform.transform) as u16,
				len: 1,
			},
			UniformField {
				name: "u_pattern",
				ty: UniformType::Mat3x2 { layout: MatrixLayout::RowMajor },
				offset: dataview::offset_of!(ColorUniform.pattern) as u16,
				len: 1,
			},
			UniformField {
				name: "u_colormod",
				ty: UniformType::F4,
				offset: dataview::offset_of!(ColorUniform.colormod) as u16,
				len: 1,
			},
		],
	};
}
