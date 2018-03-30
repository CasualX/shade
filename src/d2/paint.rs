use std::{cmp, slice};

use {Allocate, ICanvas, Primitive, TShader, Index};
use super::{Point2, Rect, Rad, Color, ToVertex, ColorV, TexV, bezier2};

//----------------------------------------------------------------

/// Paint bucket.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Paint<Shader> {
	pub color1: Color,
	pub color2: Color,
	pub segments: u32,
	pub shader: Shader,
}
impl<S: Default> Default for Paint<S> {
	fn default() -> Paint<S> {
		Paint {
			color1: Color::dup(1.0),
			color2: Color::dup(1.0),
			segments: 64,
			shader: S::default(),
		}
	}
}
impl<S> ToVertex<ColorV> for Paint<S> {
	fn to_vertex(&self, pt: Point2, _index: usize) -> ColorV {
		ColorV { pt, fg: self.color1, bg: self.color2 }
	}
}
impl<S> ToVertex<TexV> for Paint<S> {
	fn to_vertex(&self, pt: Point2, _index: usize) -> TexV {
		TexV { pt, uv: pt }
	}
}

//----------------------------------------------------------------

/// Filling shapes with paint.
pub trait IPaint<S> {
	/// Fills a rectangle.
	fn fill_rect(&mut self, paint: &Paint<S>, rc: &Rect);
	/// Fills the edges with given thickness of a rectangle.
	fn fill_edge_rect(&mut self, paint: &Paint<S>, rc: &Rect, thickness: f32);
	/// Fills a rectangle with rounded edges.
	fn fill_round_rect(&mut self, paint: &Paint<S>, rc: &Rect, sx: f32, sy: f32);
	/// Fills a convex shape with given points.
	fn fill_convex(&mut self, paint: &Paint<S>, pts: &[Point2]);
	/// Fills triangles.
	fn fill_polygon(&mut self, paint: &Paint<S>, pts: &[Point2], triangles: &[(Index, Index, Index)]);
	/// Fills a quad.
	fn fill_quad(&mut self, paint: &Paint<S>, top_left: &Point2, top_right: &Point2, bottom_left: &Point2, bottom_right: &Point2);
	/// Fills an ellipse.
	fn fill_ellipse(&mut self, paint: &Paint<S>, rc: &Rect);
	/// Fills a pie.
	fn fill_pie(&mut self, paint: &Paint<S>, rc: &Rect, start: Rad, sweep: Rad);
	/// Fills a ring.
	fn fill_ring(&mut self, paint: &Paint<S>, rc: &Rect, width: f32);
	/// Fills the area between the pivot point and a quadratic bezier curve.
	fn fill_bezier2(&mut self, paint: &Paint<S>, pivot: &Point2, pts: &[Point2; 3]);
}

impl<S: TShader, C: ICanvas> IPaint<S> for C
	where C::Buffers: Allocate<S::Vertex>,
	      Paint<S>: ToVertex<S::Vertex>,
{
	fn fill_rect(&mut self, paint: &Paint<S>, rc: &Rect) {
		// 4 vertices, 2 primitives, 6 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			paint.to_vertex(rc.top_left(), 0),
			paint.to_vertex(rc.top_right(), 1),
			paint.to_vertex(rc.bottom_right(), 2),
			paint.to_vertex(rc.bottom_left(), 3),
		);
	}
	fn fill_edge_rect(&mut self, paint: &Paint<S>, rc: &Rect, thickness: f32) {
		// 8 vertices, 8 primitives, 24 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 5, 4, 0, 1, 5,
			1, 6, 5, 1, 2, 6,
			2, 7, 6, 2, 3, 7,
			3, 4, 7, 3, 0, 4;
			paint.to_vertex(rc.top_left(), 0),
			paint.to_vertex(rc.top_right(), 1),
			paint.to_vertex(rc.bottom_right(), 2),
			paint.to_vertex(rc.bottom_left(), 3),
			paint.to_vertex(rc.top_left() + Point2(thickness, thickness), 4),
			paint.to_vertex(rc.top_right() + Point2(-thickness, thickness), 5),
			paint.to_vertex(rc.bottom_right() + Point2(-thickness, -thickness), 6),
			paint.to_vertex(rc.bottom_left() + Point2(thickness, -thickness), 7),
		);
	}
	fn fill_round_rect(&mut self, paint: &Paint<S>, rc: &Rect, sx: f32, sy: f32) {
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
	fn fill_convex(&mut self, paint: &Paint<S>, pts: &[Point2]) {
		// Degenerate convex shape
		if pts.len() < 3 {
			return;
		}
		// n vertices, n - 2 primitives, (n - 2) * 3 indices
		let n = pts.len() - 2;
		let (vp, ip) = draw_primitive::<C, S>(self, pts.len(), n);
		// Add indices
		for i in 0..n {
			ip[i].0 += i as Index;
			ip[i].1 += (i + 1) as Index;
			ip[i].2 += (n + 1 - i) as Index;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = paint.to_vertex(pts[v], v);
		}
	}
	fn fill_polygon(&mut self, paint: &Paint<S>, pts: &[Point2], triangles: &[(Index, Index, Index)]) {
		let (vp, ip) = draw_primitive::<C, S>(self, pts.len(), triangles.len());
		// Add indices
		for i in 0..triangles.len() {
			// Must be indices into the points slice
			let (p1, p2, p3) = triangles[i];
			let _ = pts[p1 as usize];
			let _ = pts[p2 as usize];
			let _ = pts[p3 as usize];
			ip[i].0 += p1;
			ip[i].1 += p2;
			ip[i].2 += p3;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = paint.to_vertex(pts[v], v);
		}
	}
	fn fill_quad(&mut self, paint: &Paint<S>, top_left: &Point2, top_right: &Point2, bottom_left: &Point2, bottom_right: &Point2) {
		// 4 vertices, 2 primitives, 6 indices
		draw_primitive!(
			self;
			Primitive::Triangles;
			0, 1, 2, 0, 2, 3;
			paint.to_vertex(*top_left, 0),
			paint.to_vertex(*top_right, 1),
			paint.to_vertex(*bottom_right, 2),
			paint.to_vertex(*bottom_left, 3),
		);
	}
	fn fill_ellipse(&mut self, paint: &Paint<S>, rc: &Rect) {
		// n + 1 vertices, n primitives, n * 3 indices
		let n = cmp::max(3, paint.segments) as usize;
		let (vp, ip) = draw_primitive::<C, S>(self, n + 1, n);

		// Add indices
		for i in 0..n - 1 {
			ip[i].1 += (i + 1) as Index;
			ip[i].2 += (i + 2) as Index;
		}
		ip[n - 1].1 += n as Index;
		ip[n - 1].2 += 1;

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		vp[0] = paint.to_vertex(center, 0);
		for v in 1..n + 1 {
			vp[v] = paint.to_vertex(pt * radius + center, v);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn fill_pie(&mut self, paint: &Paint<S>, rc: &Rect, start: Rad, sweep: Rad) {
		if sweep <= -Rad::turn() || sweep >= Rad::turn() {
			return self.fill_ellipse(paint, rc);
		}

		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(paint.segments, 2) as usize;
		let (vp, ip) = draw_primitive::<C, S>(self, n + 2, n);

		// Add indices
		for i in 0..n {
			ip[i].1 += (i + 1) as Index;
			ip[i].2 += (i + 2) as Index;
		}

		// Precompute trigs
		let (s, c) = (sweep / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = {
			let (y, x) = start.sin_cos();
			Point2(x, y)
		};

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for v in 0..n {
			vp[v] = paint.to_vertex(pt * radius + center, v);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn fill_ring(&mut self, paint: &Paint<S>, rc: &Rect, width: f32) {
		// n * 2 vertices, n * 2 primitives, n * 6 indices
		let n = cmp::max(3, paint.segments) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Triangles, n * 2, n * 2);

		// Add indices
		for i in 0..n - 1 {
			let v = i * 2;
			let i = i * 6;
			ip[i] += v as Index;
			ip[i + 1] += (v + 1) as Index;
			ip[i + 2] += (v + 2) as Index;
			ip[i + 3] += (v + 1) as Index;
			ip[i + 4] += (v + 2) as Index;
			ip[i + 5] += (v + 3) as Index;
		}
		{
			let v = (n - 1) * 2;
			let i = (n - 1) * 6;
			ip[i] += v as Index;
			ip[i + 1] += (v + 1) as Index;
			// ip[i + 2] += 0;
			ip[i + 3] += (v + 1) as Index;
			// ip[i + 4] += 0;
			ip[i + 5] += 1;
		}

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let width = radius - Point2::dup(width);
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for v in 0..n {
			let v = v * 2;
			vp[v] = paint.to_vertex(pt * radius + center, v);
			vp[v + 1] = paint.to_vertex(pt * (radius - width) + center, v + 1);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn fill_bezier2(&mut self, paint: &Paint<S>, pivot: &Point2, pts: &[Point2; 3]) {
		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(paint.segments, 2) as usize;
		let (vp, ip) = draw_primitive::<C, S>(self, n + 2, n);

		// Add indices
		for i in 0..n {
			ip[i].1 += (i + 1) as Index;
			ip[i].2 += (i + 2) as Index;
		}

		// Add vertices
		vp[0] = paint.to_vertex(*pivot, 0);
		for v in 1..n + 2 {
			let pt = bezier2(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2]);
			vp[v] = paint.to_vertex(pt, v);
		}
	}
}

// Help the compiler eliminate bounds checks
#[inline(always)]
fn draw_primitive<C: ICanvas, S: TShader>(cv: &mut C, nverts: usize, nprims: usize) -> (&mut [S::Vertex], &mut [(Index, Index, Index)])
	where C::Buffers: Allocate<S::Vertex>
{
	let (vp, ip) = cv.draw_primitive::<S>(Primitive::Triangles, nverts, nprims);
	let verts = &mut vp[..nverts];
	let indices = &mut ip[..nprims * 3];
	let indices = unsafe { slice::from_raw_parts_mut(indices.as_mut_ptr() as _, nprims) };
	(verts, indices)
}
