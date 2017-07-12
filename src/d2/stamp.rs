
use ::{Shader, Primitive};
use super::{Point, Rect, ToVertex, TexV};

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stamp {
	pub uv: Rect,
}
impl ToVertex<TexV> for Stamp {
	fn to_vertex(&self, pt: Point) -> TexV {
		TexV { pt, uv: pt }
	}
	fn to_vertex_uv(&self, pt: Point, uv: Point) -> TexV {
		TexV { pt, uv }
	}
}

//----------------------------------------------------------------

pub trait IStamp {
	fn stamp_rect(&mut self, stamp: &Stamp, rc: &Rect);
	fn stamp_quad(&mut self, stamp: &Stamp, origin: &Point, x: &Point, y: &Point);
}

impl<'a, T: Shader<'a>> IStamp for T where Stamp: ToVertex<T::Vertex> {
	fn stamp_rect(&mut self, stamp: &Stamp, rc: &Rect) {
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			stamp.to_vertex_uv(rc.top_left(), stamp.uv.top_left()),
			stamp.to_vertex_uv(rc.top_right(), stamp.uv.top_right()),
			stamp.to_vertex_uv(rc.bottom_right(), stamp.uv.bottom_right()),
			stamp.to_vertex_uv(rc.bottom_left(), stamp.uv.bottom_left()),
		);
	}
	fn stamp_quad(&mut self, stamp: &Stamp, origin: &Point, x: &Point, y: &Point) {
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			stamp.to_vertex_uv(*origin, stamp.uv.top_left()),
			stamp.to_vertex_uv(*origin + *x, stamp.uv.top_right()),
			stamp.to_vertex_uv(*origin + *y + *x, stamp.uv.bottom_right()),
			stamp.to_vertex_uv(*origin + *y, stamp.uv.bottom_left()),
		);
	}
}
