use super::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MockVertex {
	pos: Point2<f32>,
}

unsafe impl TVertex for MockVertex {
	const VERTEX_LAYOUT: &'static VertexLayout = &VertexLayout {
		size: std::mem::size_of::<MockVertex>() as u16,
		alignment: std::mem::align_of::<MockVertex>() as u16,
		attributes: &[
			VertexAttribute {
				format: VertexAttributeFormat::F32,
				len: 2,
				offset: 0,
			},
		],
	};
}

impl ToVertex<MockVertex> for () {
	#[inline]
	fn to_vertex(&self, pos: Point2<f32>, _index: usize) -> MockVertex {
		MockVertex { pos }
	}
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MockUniform {}

unsafe impl TUniform for MockUniform {
	const UNIFORM_LAYOUT: &'static UniformLayout = &UniformLayout {
		size: std::mem::size_of::<MockUniform>() as u16,
		alignment: std::mem::align_of::<MockUniform>() as u16,
		attributes: &[],
	};
}

mod pen;
mod paint;
mod stamp;
