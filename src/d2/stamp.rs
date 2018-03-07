
use {Shader, Primitive};
use super::{Point2, Vec2, Rect, ToVertex, TexV};

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stamp {
	pub uv: Rect,
}
impl ToVertex<TexV> for Stamp {
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

pub trait IStamp {
	fn stamp_rect(&mut self, stamp: &Stamp, rc: &Rect);
	fn stamp_quad(&mut self, stamp: &Stamp, origin: &Point2, x: &Vec2, y: &Vec2);
}

impl<S: Shader> IStamp for S where Stamp: ToVertex<S::Vertex> {
	fn stamp_rect(&mut self, stamp: &Stamp, rc: &Rect) {
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
	fn stamp_quad(&mut self, stamp: &Stamp, origin: &Point2, x: &Vec2, y: &Vec2) {
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
