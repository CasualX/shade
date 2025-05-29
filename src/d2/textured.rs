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
		size: std::mem::size_of::<TexturedVertex>() as u16,
		alignment: std::mem::align_of::<TexturedVertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "a_pos",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TexturedVertex.pos) as u16,
			},
			VertexAttribute {
				name: "a_texcoord",
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
#[derive(Copy, Clone, Debug, dataview::Pod)]
#[repr(C)]
pub struct TexturedUniform {
	pub transform: Transform2f,
	pub texture: Texture2D,
	pub colormod: Vec4f,
}

impl Default for TexturedUniform {
	fn default() -> Self {
		TexturedUniform {
			transform: Transform2::IDENTITY,
			texture: Texture2D::INVALID,
			colormod: Vec4::dup(1.0f32),
		}
	}
}

unsafe impl TUniform for TexturedUniform {
	const LAYOUT: &'static UniformLayout = &UniformLayout {
		size: std::mem::size_of::<TexturedUniform>() as u16,
		alignment: std::mem::align_of::<TexturedUniform>() as u16,
		fields: &[
			UniformField {
				name: "u_transform",
				ty: UniformType::Mat3x2 { order: MatrixLayout::RowMajor },
				offset: dataview::offset_of!(TexturedUniform.transform) as u16,
				len: 1,
			},
			UniformField {
				name: "u_texture",
				ty: UniformType::Sampler2D,
				offset: dataview::offset_of!(TexturedUniform.texture) as u16,
				len: 1,
			},
			UniformField {
				name: "u_colormod",
				ty: UniformType::F4,
				offset: dataview::offset_of!(TexturedUniform.colormod) as u16,
				len: 1,
			},
		],
	};
}
