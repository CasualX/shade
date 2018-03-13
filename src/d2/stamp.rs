
use {Allocate, ICanvas, TShader, Primitive};
use super::{Point2, Vec2, Rect, ToVertex, TexV};

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stamp<S> {
	pub uv: Rect,
	pub shader: S,
}
impl<S> ToVertex<TexV> for Stamp<S> {
	fn to_vertex(&self, pt: Point2, index: usize) -> TexV {
		let uv = match index {
			0 => self.uv.top_left(),
			1 => self.uv.top_right(),
			2 => self.uv.bottom_right(),
			3 => self.uv.bottom_left(),
			_ => pt,
		};
		TexV { pt, uv }
	}
}

//----------------------------------------------------------------

pub trait IStamp<S> {
	fn stamp_rect(&mut self, stamp: &Stamp<S>, rc: &Rect);
	fn stamp_quad(&mut self, stamp: &Stamp<S>, origin: &Point2, x: &Vec2, y: &Vec2);
}

impl<S: TShader, C: ICanvas> IStamp<S> for C
	where C::Buffers: Allocate<S::Vertex>,
	      Stamp<S>: ToVertex<S::Vertex>,
{
	fn stamp_rect(&mut self, stamp: &Stamp<S>, rc: &Rect) {
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			stamp.to_vertex(rc.top_left(), 0),
			stamp.to_vertex(rc.top_right(), 1),
			stamp.to_vertex(rc.bottom_right(), 2),
			stamp.to_vertex(rc.bottom_left(), 3),
		);
	}
	fn stamp_quad(&mut self, stamp: &Stamp<S>, origin: &Point2, x: &Vec2, y: &Vec2) {
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			stamp.to_vertex(*origin, 0),
			stamp.to_vertex(*origin + *x, 1),
			stamp.to_vertex(*origin + *y + *x, 2),
			stamp.to_vertex(*origin + *y, 3),
		);
	}
}
