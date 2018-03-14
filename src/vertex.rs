/*!
Vertex and Index types.
*/

/// Marker trait for vertex types.
pub trait TVertex: Copy + 'static {
	fn vertex_uid() -> u32;
}

/// Index type for indexed drawing.
pub type Index = u32;

/// Vertex placeholder.
pub enum PlaceV {}
