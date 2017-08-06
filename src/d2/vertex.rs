
use ::vertex::IVertex;
use super::{Point2, Color};

pub trait ToVertex<V> {
	fn to_vertex(&self, pt: Point2) -> V;
	fn to_vertex_uv(&self, pt: Point2, _uv: Point2) -> V { self.to_vertex(pt) }
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TexV {
	pub pt: Point2,
	pub uv: Point2,
}
impl IVertex for TexV {
	fn uid() -> u32 { 0x7c88545a }
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ColorV {
	pub pt: Point2,
	pub fg: Color,
	pub bg: Color,
}
impl IVertex for ColorV {
	fn uid() -> u32 { 0xa1c184ae }
}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TextV {
	pub pt: Point2,
	pub uv: Point2,
	pub fg: Color,
	pub bg: Color,
}
impl IVertex for TextV {
	fn uid() -> u32 { 0x0d92e32e }
}
