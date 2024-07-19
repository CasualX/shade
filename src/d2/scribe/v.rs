use super::*;

/// Text vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TextVertex {
	pub pos: Vec2<f32>,
	pub uv: Vec2<f32>,
	pub color: Vec4<u8>,
	pub outline: Vec4<u8>,
}

unsafe impl TVertex for TextVertex {
	const VERTEX_LAYOUT: &'static crate::VertexLayout = &crate::VertexLayout {
		size: std::mem::size_of::<TextVertex>() as u16,
		alignment: std::mem::align_of::<TextVertex>() as u16,
		attributes: &[
			VertexAttribute {
				format: VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(TextVertex.pos) as u16,
			},
			VertexAttribute {
				format: VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(TextVertex.uv) as u16,
			},
			VertexAttribute {
				format: VertexAttributeFormat::U8Norm,
				len: 4,
				offset: dataview::offset_of!(TextVertex.color) as u16,
			},
			VertexAttribute {
				format: VertexAttributeFormat::U8Norm,
				len: 4,
				offset: dataview::offset_of!(TextVertex.outline) as u16,
			},
		],
	};
}

impl ToVertex<TextVertex> for TextVertex {
	#[inline]
	fn to_vertex(&self, pos: Vec2<f32>, _index: usize) -> TextVertex {
		TextVertex { pos, ..*self }
	}
}
