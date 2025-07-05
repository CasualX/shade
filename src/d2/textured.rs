use super::*;

/// Textured vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TexturedVertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
	pub color: Vec4<u8>,
}

unsafe impl TVertex for TexturedVertex {
	const LAYOUT: &'static crate::VertexLayout = &crate::VertexLayout {
		size: mem::size_of::<TexturedVertex>() as u16,
		alignment: mem::align_of::<TexturedVertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "a_pos",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TexturedVertex.pos) as u16,
			},
			VertexAttribute {
				name: "a_uv",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TexturedVertex.uv) as u16,
			},
			VertexAttribute {
				name: "a_color",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TexturedVertex.color) as u16,
			},
		],
	};
}

/// Textured template.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TexturedTemplate {
	pub uv: Vec2f,
	pub color: Vec4<u8>,
}

impl ToVertex<TexturedVertex> for TexturedTemplate {
	#[inline]
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> TexturedVertex {
		TexturedVertex { pos, uv: self.uv, color: self.color }
	}
}

/// Textured uniform.
#[derive(Clone, Debug, PartialEq)]
pub struct TexturedUniform {
	pub transform: Transform2f,
	pub texture: Texture2D,
	pub colormod: Vec4f,
}

impl Default for TexturedUniform {
	#[inline]
	fn default() -> Self {
		TexturedUniform {
			transform: Transform2::IDENTITY,
			texture: Texture2D::INVALID,
			colormod: Vec4::ONE,
		}
	}
}

impl UniformVisitor for TexturedUniform {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
		set.value("u_colormod", &self.colormod);
	}
}

/// DrawBuffer for textured graphics.
pub type TexturedBuffer = DrawBuffer<TexturedVertex, TexturedUniform>;
