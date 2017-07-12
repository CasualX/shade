use ::std::cmp;

use ::{Primitive, Shader, Index};
use super::{Point, Rect, Rad, Color, ToVertex, ColorV, TexV, bezier2};

//----------------------------------------------------------------

/// Paint bucket.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Paint {
	pub color1: Color,
	pub color2: Color,
	pub segments: u32,
	_private: (),
}
impl Default for Paint {
	fn default() -> Paint {
		Paint {
			color1: Color::dup(1.0),
			color2: Color::dup(1.0),
			segments: 64,
			_private: (),
		}
	}
}
impl ToVertex<ColorV> for Paint {
	fn to_vertex(&self, pt: Point) -> ColorV {
		ColorV { pt, fg: self.color1, bg: self.color2 }
	}
}
impl ToVertex<TexV> for Paint {
	fn to_vertex(&self, pt: Point) -> TexV {
		TexV { pt, uv: pt }
	}
}

//----------------------------------------------------------------

pub trait IPaint {
	/// Fills a rectangle.
	fn fill_rect(&mut self, paint: &Paint, rc: &Rect);
	/// Fills the edges with given thickness of a rectangle.
	fn fill_edge_rect(&mut self, paint: &Paint, rc: &Rect, thickness: f32);
	/// Fills a rectangle with rounded edges.
	fn fill_round_rect(&mut self, paint: &Paint, rc: &Rect, sx: f32, sy: f32);
	/// Fills a convex shape with given points.
	fn fill_convex(&mut self, paint: &Paint, pts: &[Point]);
	/// Fills triangles.
	fn fill_polygon(&mut self, paint: &Paint, pts: &[Point], triangles: &[(Index, Index, Index)]);
	/// Fills a quad.
	fn fill_quad(&mut self, paint: &Paint, top_left: &Point, top_right: &Point, bottom_left: &Point, bottom_right: &Point);
	/// Fills an ellipse.
	fn fill_ellipse(&mut self, paint: &Paint, rc: &Rect);
	/// Fills a pie.
	fn fill_pie(&mut self, paint: &Paint, rc: &Rect, start: Rad, sweep: Rad);
	/// Fills a ring.
	fn fill_ring(&mut self, paint: &Paint, rc: &Rect, width: f32);
	/// Fills the area between the pivot point and a quadratic bezier curve.
	fn fill_bezier2(&mut self, paint: &Paint, pivot: &Point, pts: &[Point; 3]);
}

impl<'a, T: Shader<'a>> IPaint for T where Paint: ToVertex<T::Vertex> {
	fn fill_rect(&mut self, paint: &Paint, rc: &Rect) {
		// 4 vertices, 2 primitives, 6 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			paint.to_vertex(rc.top_left()),
			paint.to_vertex(rc.top_right()),
			paint.to_vertex(rc.bottom_right()),
			paint.to_vertex(rc.bottom_left()),
		);
	}
	fn fill_edge_rect(&mut self, paint: &Paint, rc: &Rect, thickness: f32) {
		// 8 vertices, 8 primitives, 24 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 5, 4, 0, 1, 5,
			1, 6, 5, 1, 2, 6,
			2, 7, 6, 2, 3, 7,
			3, 4, 7, 3, 0, 4;
			paint.to_vertex(rc.top_left()),
			paint.to_vertex(rc.top_right()),
			paint.to_vertex(rc.bottom_right()),
			paint.to_vertex(rc.bottom_left()),
			paint.to_vertex(rc.top_left() + Point::new(thickness, thickness)),
			paint.to_vertex(rc.top_right() + Point::new(-thickness, thickness)),
			paint.to_vertex(rc.bottom_right() + Point::new(-thickness, -thickness)),
			paint.to_vertex(rc.bottom_left() + Point::new(thickness, -thickness)),
		);
	}
	fn fill_round_rect(&mut self, paint: &Paint, rc: &Rect, sx: f32, sy: f32) {
		// Fixup parameters
		let sx = if sx + sx > rc.width() { rc.width() * 0.5 } else { sx };
		let sy = if sy + sy > rc.height() { rc.height() * 0.5 } else { sy };

		// Edge case: just fill a rectangle
		if sx <= 0.0 || sy <= 0.0 {
			return self.fill_rect(paint, rc);
		}

		// TODO? Do it properly so the edges are synced up
		unimplemented!()
	}
	fn fill_convex(&mut self, paint: &Paint, pts: &[Point]) {
		// Degenerate convex shape
		if pts.len() < 3 {
			return;
		}
		// n vertices, n - 2 primitives, (n - 2) * 3 indices
		let n = pts.len() - 2;
		let (vp, ip) = self.draw_primitive(Primitive::Triangles, pts.len(), n);
		// Add indices
		for i in 0..n {
			ip[i * 3] += i as Index;
			ip[i * 3 + 1] += (i + 1) as Index;
			ip[i * 3 + 2] += (n + 1 - i) as Index;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = paint.to_vertex(pts[v]);
		}
	}
	fn fill_polygon(&mut self, paint: &Paint, pts: &[Point], triangles: &[(Index, Index, Index)]) {
		let (vp, ip) = self.draw_primitive(Primitive::Triangles, pts.len(), triangles.len());
		// Add indices
		for i in 0..triangles.len() {
			// Must be indices into the points slice
			let (p1, p2, p3) = triangles[i];
			let _ = pts[p1 as usize];
			let _ = pts[p2 as usize];
			let _ = pts[p3 as usize];
			ip[i * 3] += p1;
			ip[i * 3 + 1] += p2;
			ip[i * 3 + 2] += p3;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = paint.to_vertex(pts[v]);
		}
	}
	fn fill_quad(&mut self, paint: &Paint, top_left: &Point, top_right: &Point, bottom_left: &Point, bottom_right: &Point) {
		// 4 vertices, 2 primitives, 6 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			paint.to_vertex(*top_left),
			paint.to_vertex(*top_right),
			paint.to_vertex(*bottom_right),
			paint.to_vertex(*bottom_left),
		);
	}
	fn fill_ellipse(&mut self, paint: &Paint, rc: &Rect) {
		// n + 1 vertices, n primitives, n * 3 indices
		let n = ((paint.segments & !3) + 4) as usize;
		let (vp, ip) = self.draw_primitive(Primitive::Triangles, n + 1, n);

		// Add indices
		for i in 0..n - 1 {
			ip[i * 3 + 1] += (i + 1) as Index;
			ip[i * 3 + 2] += (i + 2) as Index;
		}
		ip[n * 3 - 2] += n as Index;
		ip[n * 3 - 1] += 1;

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = Point::new(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		vp[0] = paint.to_vertex(center);
		for i in 1..n + 1 {
			vp[i] = paint.to_vertex(pt * radius + center);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn fill_pie(&mut self, paint: &Paint, rc: &Rect, start: Rad, sweep: Rad) {
		if sweep <= -Rad::turn() || sweep >= Rad::turn() {
			return self.fill_ellipse(paint, rc);
		}

		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(paint.segments, 2) as usize;
		let (vp, ip) = self.draw_primitive(Primitive::Triangles, n + 2, n);

		// Add indices
		for i in 0..n {
			ip[i * 3 + 1] += (i + 1) as Index;
			ip[i * 3 + 2] += (i + 2) as Index;
		}

		// Precompute trigs
		let (s, c) = (sweep / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = {
			let (y, x) = start.sin_cos();
			Point::new(x, y)
		};

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for i in 0..n {
			vp[i] = paint.to_vertex(pt * radius + center);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn fill_ring(&mut self, paint: &Paint, rc: &Rect, width: f32) { unimplemented!() }
	fn fill_bezier2(&mut self, paint: &Paint, pivot: &Point, pts: &[Point; 3]) {
		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(paint.segments, 2) as usize;
		let (vp, ip) = self.draw_primitive(Primitive::Lines, n + 2, n);

		// Add indices
		for i in 0..n {
			ip[i * 3 + 1] += (i + 1) as Index;
			ip[i * 3 + 2] += (i + 2) as Index;
		}

		// Add vertices
		vp[0] = paint.to_vertex(*pivot);
		for i in 1..n + 2 {
			let pt = bezier2(i as i32 as f32 / n as i32 as f32, pts);
			vp[i] = paint.to_vertex(pt);
		}
	}
}
