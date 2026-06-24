use super::*;

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TexturedVertexN {
	pub pos: Vec3f,
	pub normal: Vec3f,
	pub uv: Vec2f,
}

unsafe impl TVertex for TexturedVertexN {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<TexturedVertexN>() as u16,
		alignment: mem::align_of::<TexturedVertexN>() as u16,
		attributes: &[
			VertexAttribute { name: "a_pos", format: VertexAttributeFormat::F32v3, offset: dataview::offset_of!(TexturedVertexN.pos) as u16 },
			VertexAttribute { name: "a_normal", format: VertexAttributeFormat::F32v3, offset: dataview::offset_of!(TexturedVertexN.normal) as u16 },
			VertexAttribute { name: "a_uv", format: VertexAttributeFormat::F32v2, offset: dataview::offset_of!(TexturedVertexN.uv) as u16 },
		],
	};
}

impl TVertex3 for TexturedVertexN {
	#[inline]
	fn position(&self) -> cvmath::Vec3<f32> {
		self.pos
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexturedUniform3<'a> {
	pub transform: Mat4f,
	pub texture: &'a dyn Texture2D,
	pub colormod: Vec4f,
}

impl<'a> Default for TexturedUniform3<'a> {
	#[inline]
	fn default() -> Self {
		TexturedUniform3 {
			transform: Mat4::IDENTITY,
			texture: &crate::DefaultTexture2D,
			colormod: Vec4::ONE,
		}
	}
}

impl<'a> UniformVisitor for TexturedUniform3<'a> {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", self.texture);
		set.value("u_colorModulation", &self.colormod);
	}
}

unsafe impl<'a> TUniformKey for TexturedUniform3<'a> {
	#[inline]
	fn key() -> any::TypeId {
		any::TypeId::of::<TexturedUniform3<'static>>()
	}
}
