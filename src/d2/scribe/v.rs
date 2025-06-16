use super::*;

/// Text vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TextVertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
	pub color: Vec4<u8>,
	pub outline: Vec4<u8>,
}

unsafe impl TVertex for TextVertex {
	const LAYOUT: &'static crate::VertexLayout = &crate::VertexLayout {
		size: mem::size_of::<TextVertex>() as u16,
		alignment: mem::align_of::<TextVertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "a_pos",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TextVertex.pos) as u16,
			},
			VertexAttribute {
				name: "a_texcoord",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TextVertex.uv) as u16,
			},
			VertexAttribute {
				name: "a_color",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TextVertex.color) as u16,
			},
			VertexAttribute {
				name: "a_outline",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TextVertex.outline) as u16,
			},
		],
	};
}

// Text template.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TextTemplate {
	pub uv: Vec2f,
	pub color: Vec4<u8>,
	pub outline: Vec4<u8>,
}

impl ToVertex<TextVertex> for TextTemplate {
	#[inline]
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> TextVertex {
		TextVertex { pos, uv: self.uv, color: self.color, outline: self.outline }
	}
}

/// Text uniform.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TextUniform {
	pub transform: Transform2f,
	pub texture: Texture2D,
	pub unit_range: Vec2f,
	pub threshold: f32,
	pub out_bias: f32,
	pub outline_width_absolute: f32,
	pub outline_width_relative: f32,
	pub gamma: f32,
}

impl Default for TextUniform {
	fn default() -> Self {
		TextUniform {
			transform: Transform2::IDENTITY,
			texture: Texture2D::INVALID,
			unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
			threshold: 0.5,
			out_bias: 0.0,
			outline_width_absolute: 1.0,
			outline_width_relative: 0.125,
			gamma: 1.0,
		}
	}
}

unsafe impl TUniform for TextUniform {
	const LAYOUT: &'static UniformLayout = &UniformLayout {
		size: mem::size_of::<TextUniform>() as u16,
		alignment: mem::align_of::<TextUniform>() as u16,
		fields: &[
			UniformField {
				name: "u_transform",
				ty: UniformType::Mat3x2 { layout: MatrixLayout::RowMajor },
				offset: dataview::offset_of!(TextUniform.transform) as u16,
				len: 1,
			},
			UniformField {
				name: "u_texture",
				ty: UniformType::Sampler2D,
				offset: dataview::offset_of!(TextUniform.texture) as u16,
				len: 1,
			},
			UniformField {
				name: "u_unit_range",
				ty: UniformType::F2,
				offset: dataview::offset_of!(TextUniform.unit_range) as u16,
				len: 1,
			},
			UniformField {
				name: "u_threshold",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.threshold) as u16,
				len: 1,
			},
			UniformField {
				name: "u_out_bias",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.out_bias) as u16,
				len: 1,
			},
			UniformField {
				name: "u_outline_width_absolute",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.outline_width_absolute) as u16,
				len: 1,
			},
			UniformField {
				name: "u_outline_width_relative",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.outline_width_relative) as u16,
				len: 1,
			},
			UniformField {
				name: "u_gamma",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.gamma) as u16,
				len: 1,
			},
		],
	};
}
