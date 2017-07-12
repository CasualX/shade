/*!
2D vertices.
*/

use ::vertex::IVertex;
use super::{Point, Color};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TexV {
	pub pt: Point,
	pub uv: Point,
}
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ColorV {
	pub pt: Point,
	pub fg: Color,
	pub bg: Color,
}
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TextV {
	pub pt: Point,
	pub uv: Point,
	pub fg: Color,
	pub bg: Color,
}

impl IVertex for TexV {
	fn uid() -> u32 { 0x7c88545a }
}
impl IVertex for ColorV {
	fn uid() -> u32 { 0xa1c184ae }
}
impl IVertex for TextV {
	fn uid() -> u32 { 0x0d92e32e }
}

pub trait ToVertex<V> {
	fn to_vertex(&self, pt: Point) -> V;
	fn to_vertex_uv(&self, pt: Point, _uv: Point) -> V { self.to_vertex(pt) }
}
