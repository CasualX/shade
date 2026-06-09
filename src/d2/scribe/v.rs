use super::*;

/// Text vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TextVertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
	pub color: Vec4<u8>,
	pub outline: Vec4<u8>,
	pub data: TextVertexData,
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
				name: "a_uv",
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
			VertexAttribute {
				name: "a_data",
				format: VertexAttributeFormat::I32,
				offset: dataview::offset_of!(TextVertex.data) as u16,
			},
		],
	};
}

/// Metadata for a [`TextVertex`].
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct TextVertexData(pub i32);

unsafe impl dataview::Pod for TextVertexData {}

impl TextVertexData {
	pub const NONE: TextVertexData = TextVertexData(0);

	/// Top left corner of a glyph quad.
	pub const TOP_LEFT: TextVertexData = TextVertexData(0b00);
	/// Top right corner of a glyph quad.
	pub const TOP_RIGHT: TextVertexData = TextVertexData(0b01);
	/// Bottom left corner of a glyph quad.
	pub const BOTTOM_LEFT: TextVertexData = TextVertexData(0b10);
	/// Bottom right corner of a glyph quad.
	pub const BOTTOM_RIGHT: TextVertexData = TextVertexData(0b11);
}

/// Text template.
#[derive(Clone, Debug)]
pub struct TextTemplate {
	pub uv: Vec2f,
	pub color: Vec4<u8>,
	pub outline: Vec4<u8>,
}

impl ToVertex<TextVertex> for TextTemplate {
	#[inline]
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> TextVertex {
		TextVertex { pos, uv: self.uv, color: self.color, outline: self.outline, data: TextVertexData::NONE }
	}
}

/// Text uniform.
#[derive(Clone, Debug, PartialEq)]
pub struct TextUniform {
	pub transform: Transform2f,
	pub texture: Texture2D,
	pub unit_range: Vec2f,
	pub threshold: f32,
	pub out_bias: f32,
	pub outline_width_absolute: f32,
	pub outline_width_relative: f32,
}

impl Default for TextUniform {
	#[inline]
	fn default() -> Self {
		TextUniform {
			transform: Transform2::IDENTITY,
			texture: Texture2D::INVALID,
			unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
			threshold: 0.5,
			out_bias: 0.0,
			outline_width_absolute: 1.0,
			outline_width_relative: 0.125,
		}
	}
}

impl UniformVisitor for TextUniform {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
		set.value("u_unitRange", &self.unit_range);
		set.value("u_threshold", &self.threshold);
		set.value("u_outBias", &self.out_bias);
		set.value("u_outlineWidthAbsolute", &self.outline_width_absolute);
		set.value("u_outlineWidthRelative", &self.outline_width_relative);
	}
}

/// Text uniform for drawing 2D text vertices on a 3D plane.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextUniform3 {
	pub text: TextUniform,
	pub camera_transform: Mat4f,
	pub plane_transform: Transform3f,
}

impl UniformVisitor for TextUniform3 {
	fn visit(&self, set: &mut dyn UniformSetter) {
		self.text.visit(set);
		set.value("u_cameraTransform", &self.camera_transform);
		set.value("u_planeTransform", &self.plane_transform);
	}
}
