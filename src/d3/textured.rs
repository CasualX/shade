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
