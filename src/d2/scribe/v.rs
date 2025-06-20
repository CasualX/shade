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

impl UniformVisitor for TextUniform {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
		set.value("u_unit_range", &self.unit_range);
		set.value("u_threshold", &self.threshold);
		set.value("u_out_bias", &self.out_bias);
		set.value("u_outline_width_absolute", &self.outline_width_absolute);
		set.value("u_outline_width_relative", &self.outline_width_relative);
		set.value("u_gamma", &self.gamma);
	}
}
