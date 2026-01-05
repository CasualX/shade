use super::*;

#[derive(Clone, Debug)]
pub struct VertexMesh {
	pub origin: Vec3f,
	pub bounds: Bounds3f,
	pub vertices: VertexBuffer,
	pub vertices_len: u32,
}

#[derive(Clone, Debug)]
pub struct VertexIndexedMesh {
	pub origin: Vec3f,
	pub bounds: Bounds3f,
	pub vertices: VertexBuffer,
	pub vertices_len: u32,
	pub indices: IndexBuffer,
	pub indices_len: u32,
}
