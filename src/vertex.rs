/*!
*/

pub trait IVertex: Copy + 'static {
	fn uid() -> u32;
}

pub type Index = u32;

/// Vertex placeholder.
pub enum PlaceV {}
