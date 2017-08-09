
use ::{Shader, Primitive};
use super::{Point2, Vec2, Rect, ToVertex, TexV};

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stamp {
	pub uv: Rect,
}
impl ToVertex<TexV> for Stamp {
	fn to_vertex(&self, pt: Point2) -> TexV {
		TexV { pt, uv: pt }
	}
	fn to_vertex_uv(&self, pt: Point2, uv: Point2) -> TexV {
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
			stamp.to_vertex_uv(rc.top_left(), stamp.uv.top_left()),
			stamp.to_vertex_uv(rc.top_right(), stamp.uv.top_right()),
			stamp.to_vertex_uv(rc.bottom_right(), stamp.uv.bottom_right()),
			stamp.to_vertex_uv(rc.bottom_left(), stamp.uv.bottom_left()),
		);
	}
	fn stamp_quad(&mut self, stamp: &Stamp, origin: &Point2, x: &Vec2, y: &Vec2) {
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
