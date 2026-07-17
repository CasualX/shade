use super::*;

pub struct VertexMesh {
	pub origin: Vec3f,
	pub bounds: Bounds3f,
	pub vertices: Box<dyn VertexBuffer>,
	pub vertices_len: u32,
}

impl VertexMesh {
	/// Creates a new instance from the given vertices.
	#[inline]
	pub fn new<V: TVertex3>(g: &mut dyn IGraphics, origin: Vec3f, vertices: &[V], usage: BufferUsage) -> VertexMesh {
		let vertex_buffer = g.vertex_buffer(vertices, usage);
		VertexMesh {
			origin,
			bounds: compute_bounds(vertices),
			vertices: vertex_buffer,
			vertices_len: vertices.len() as u32,
		}
	}
}

pub struct VertexIndexedMesh {
	pub origin: Vec3f,
	pub bounds: Bounds3f,
	pub vertices: Box<dyn VertexBuffer>,
	pub vertices_len: u32,
	pub indices: Box<dyn IndexBuffer>,
	pub indices_len: u32,
}
