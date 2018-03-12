/*!
Vertex and Index types.
*/

/// Marker trait for vertex types.
pub trait IVertex: Copy + 'static {
	fn uid() -> u32;
}

/// Index type for indexed drawing.
pub type Index = u32;

/// Vertex placeholder.
pub enum PlaceV {}

/// Vertex buffers
pub trait VertexBuffer<V: IVertex> {
	/// Allocates a number of uninitialized vertices.
	fn allocate(&mut self, n: usize) -> &mut [V];
}
