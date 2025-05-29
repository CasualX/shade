use super::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MockVertex {
	pos: Point2f,
}

unsafe impl TVertex for MockVertex {
	const LAYOUT: &'static VertexLayout = &VertexLayout {
		size: mem::size_of::<MockVertex>() as u16,
		alignment: mem::align_of::<MockVertex>() as u16,
		attributes: &[
			VertexAttribute {
				name: "position",
				format: VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(MockVertex.pos) as u16,
			},
		],
	};
}

impl ToVertex<MockVertex> for () {
	#[inline]
	fn to_vertex(&self, pos: Point2f, _index: usize) -> MockVertex {
		MockVertex { pos }
	}
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MockUniform {}

unsafe impl TUniform for MockUniform {
	const LAYOUT: &'static UniformLayout = &UniformLayout {
		size: mem::size_of::<MockUniform>() as u16,
		alignment: mem::align_of::<MockUniform>() as u16,
		fields: &[],
	};
}

mod pen;
mod paint;
mod stamp;
