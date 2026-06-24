use super::*;

/// Color vertex.
#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct ColorVertex {
	pub pos: Point2f,
	pub color1: Vec4<u8>,
	pub color2: Vec4<u8>,
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
				name: "a_color1",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(ColorVertex.color1) as u16,
			},
			VertexAttribute {
				name: "a_color2",
				format: VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(ColorVertex.color2) as u16,
			},
		],
	};
}

/// Color template.
#[derive(Copy, Clone, Debug)]
pub struct ColorTemplate {
	pub color1: Vec4<u8>,
	pub color2: Vec4<u8>,
}

impl ToVertex<ColorVertex> for ColorTemplate {
	#[inline]
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> ColorVertex {
		ColorVertex { pos, color1: self.color1, color2: self.color2 }
	}
}

/// Color uniform.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorUniform<'a> {
	pub transform: Transform2f,
	pub pattern: Transform2f,
	pub colormod: Vec4f,
	/// Gradient texture.
	pub texture: &'a dyn Texture2D,
}

impl<'a> Default for ColorUniform<'a> {
	#[inline]
	fn default() -> Self {
		ColorUniform {
			transform: Transform2f::IDENTITY,
			pattern: Transform2f::IDENTITY,
			colormod: Vec4f::ONE,
			texture: &crate::DefaultTexture2D,
		}
	}
}

impl<'a> UniformVisitor for ColorUniform<'a> {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_pattern", &self.pattern);
		set.value("u_colorModulation", &self.colormod);
		set.value("u_texture", self.texture);
	}
}

unsafe impl<'a> TUniformKey for ColorUniform<'a> {
	#[inline]
	fn key() -> any::TypeId {
		any::TypeId::of::<ColorUniform<'static>>()
	}
}

/// DrawBuilder for color rendering.
pub type ColorBuffer<'a> = DrawBuilder<'a, ColorVertex, ColorUniform<'a>>;
