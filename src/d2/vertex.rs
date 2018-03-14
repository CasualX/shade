
use vertex::TVertex;
use super::{Point2, Color};

pub trait ToVertex<V> {
	fn to_vertex(&self, pt: Point2, index: usize) -> V;
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TexV {
	pub pt: Point2,
	pub uv: Point2,
}
impl TVertex for TexV {
	fn vertex_uid() -> u32 { 0x7c88545a }
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ColorV {
	pub pt: Point2,
	pub fg: Color,
	pub bg: Color,
}
impl TVertex for ColorV {
	fn vertex_uid() -> u32 { 0xa1c184ae }
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TextV {
	pub pt: Point2,
	pub uv: Point2,
	pub fg: Color,
	pub bg: Color,
}
impl TVertex for TextV {
	fn vertex_uid() -> u32 { 0x0d92e32e }
}
