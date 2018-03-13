
use std::cmp;

use {Allocate, ICanvas, TShader, Primitive, Index};
use super::{Point2, Rect, Rad, Color, ToVertex, ColorV, TexV, bezier2, bezier3};

//----------------------------------------------------------------

/// Line drawing pencil.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pen<S> {
	pub color: Color,
	pub segments: u32,
	pub shader: S,
}
impl<S: Default> Default for Pen<S> {
	fn default() -> Pen<S> {
		Pen {
			color: Color::dup(1.0),
			segments: 64,
			shader: S::default(),
		}
	}
}
impl<S> ToVertex<ColorV> for Pen<S> {
	fn to_vertex(&self, pt: Point2, _index: usize) -> ColorV {
		ColorV { pt, fg: self.color, bg: self.color }
	}
}
impl<S> ToVertex<TexV> for Pen<S> {
	fn to_vertex(&self, pt: Point2, _index: usize) -> TexV {
		TexV { pt, uv: pt }
	}
}

//----------------------------------------------------------------

/// Line drawing functions.
pub trait IPen<S> {
	/// Draws a line from `a` to `b`.
	fn draw_line(&mut self, pen: &Pen<S>, a: Point2, b: Point2);
	/// Draws lines.
	fn draw_lines(&mut self, pen: &Pen<S>, pts: &[Point2], lines: &[(Index, Index)]);
	/// Draws a rectangle with lines.
	fn draw_line_rect(&mut self, pen: &Pen<S>, rc: &Rect);
	/// Draws a rounded rectangle with lines.
	///
	/// `sx` and `sy` are the inset in the X and Y direction respectively.
	fn draw_round_rect(&mut self, pen: &Pen<S>, rc: &Rect, sx: f32, sy: f32);
	/// Draws a line through all the points.
	///
	/// Optionally loops by drawing a line back to the start.
	fn draw_poly_line(&mut self, pen: &Pen<S>, pts: &[Point2], close: bool);
	/// Draws an ellipse touching the sides of the given rectangle.
	fn draw_ellipse(&mut self, pen: &Pen<S>, rc: &Rect);
	/// Draws an ellipse arc segment.
	fn draw_arc(&mut self, pen: &Pen<S>, rc: &Rect, start: Rad, sweep: Rad);
	/// Draws a quadratic bezier curve with given control points.
	fn draw_bezier2(&mut self, pen: &Pen<S>, pts: &[Point2; 3]);
	/// Draws a cubic bezier curve with given control points.
	fn draw_bezier3(&mut self, pen: &Pen<S>, pts: &[Point2; 4]);
	/// Draws a cubic hermite spline with given control points and tension.
	fn draw_cspline(&mut self, pen: &Pen<S>, pts: &[Point2], tension: f32);
}

//----------------------------------------------------------------

impl<S: TShader, C: ICanvas> IPen<S> for C
	where C::Buffers: Allocate<S::Vertex>,
	      Pen<S>: ToVertex<S::Vertex>,
{
	fn draw_line(&mut self, pen: &Pen<S>, a: Point2, b: Point2) {
		// 2 vertices, 1 primitive, 2 indices
		draw_primitive!(
			self;
			Primitive::Lines;
			0, 1;
			pen.to_vertex(a, 0),
			pen.to_vertex(b, 1),
		);
	}
	fn draw_lines(&mut self, pen: &Pen<S>, pts: &[Point2], lines: &[(Index, Index)]) {
		// pts.len() vertices, lines.len() primitives, lines.len() * 2 indices
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, pts.len(), lines.len());
		debug_assert_eq!(vp.len(), pts.len());
		debug_assert_eq!(ip.len(), lines.len() * 2);
		// Add indices
		for i in 0..lines.len() {
			// Must be indices into the points slice
			let (p1, p2) = lines[i];
			let _ = pts[p1 as usize];
			let _ = pts[p2 as usize];
			ip[i * 2] += p1;
			ip[i * 2 + 1] += p2;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = pen.to_vertex(pts[v], v);
		}
	}
	fn draw_line_rect(&mut self, pen: &Pen<S>, rc: &Rect) {
		// 4 vertices, 4 primitives, 8 indices
		draw_primitive!(
			self;
			Primitive::Lines;
			0, 1, 1, 2, 2, 3, 3, 0;
			pen.to_vertex(rc.top_left(), 0),
			pen.to_vertex(rc.top_right(), 1),
			pen.to_vertex(rc.bottom_right(), 2),
			pen.to_vertex(rc.bottom_left(), 3),
		);
	}
	fn draw_round_rect(&mut self, pen: &Pen<S>, rc: &Rect, sx: f32, sy: f32) {
		// Fixup parameters
		let sx = if sx + sx > rc.width() { rc.width() * 0.5 } else { sx };
		let sy = if sy + sy > rc.height() { rc.height() * 0.5 } else { sy };

		// Edge case: just draw a rectangle
		if sx <= 0.0 || sy <= 0.0 {
			return self.draw_line_rect(pen, rc);
		}

		// Draw the rounded corners as quad beziers
		// https://en.wikipedia.org/wiki/Composite_B%C3%A9zier_curve#Approximating_circular_arcs

		// TODO? Do it properly so the edges are synced up
		unimplemented!()
	}
	fn draw_poly_line(&mut self, pen: &Pen<S>, pts: &[Point2], close: bool) {
		// Degenerate polyline
		if pts.len() < 2 {
			return;
		}
		// open: n vertices, n - 1 primitives, (n - 1) * 2 indices
		// close: n vertices, n primitives, n * 2 indices
		let n = pts.len() - (!close) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, pts.len(), n);
		debug_assert_eq!(vp.len(), pts.len());
		debug_assert_eq!(ip.len(), n * 2);
		// Add indices
		for i in 0..n {
			ip[i * 2] += i as Index;
			ip[i * 2 + 1] += (i + 1) as Index;
		}
		if close {
			ip[n * 2 - 1] -= n as Index;
		}
		// Add vertices
		for v in 0..pts.len() {
			vp[v] = pen.to_vertex(pts[v], v);
		}
	}
	fn draw_ellipse(&mut self, pen: &Pen<S>, rc: &Rect) {
		// n vertices, n primitives, n * 2 indices
		let n = cmp::max(3, pen.segments) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, n, n);
		debug_assert_eq!(vp.len(), n);
		debug_assert_eq!(ip.len(), n * 2);

		// Add indices
		for i in 0..n - 1 {
			ip[i * 2] += i as Index;
			ip[i * 2 + 1] += (i + 1) as Index;
		}
		ip[n * 2 - 2] += (n - 1) as Index;

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for v in 0..n {
			vp[v] = pen.to_vertex(pt * radius + center, v);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn draw_arc(&mut self, pen: &Pen<S>, rc: &Rect, start: Rad, sweep: Rad) {
		if sweep <= -Rad::turn() || sweep >= Rad::turn() {
			return self.draw_ellipse(pen, rc);
		}

		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(pen.segments, 2) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, n + 1, n);
		debug_assert_eq!(vp.len(), n + 1);
		debug_assert_eq!(ip.len(), n * 2);

		// Add indices
		for i in 0..n {
			ip[i * 2] += i as Index;
			ip[i * 2 + 1] += (i + 1) as Index;
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
			vp[v] = pen.to_vertex(pt * radius + center, v);
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}
	fn draw_bezier2(&mut self, pen: &Pen<S>, pts: &[Point2; 3]) {
		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(pen.segments, 2) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, n + 1, n);
		debug_assert_eq!(vp.len(), n + 1);
		debug_assert_eq!(ip.len(), n * 2);

		// Add indices
		for i in 0..n {
			ip[i * 2] += i as Index;
			ip[i * 2 + 1] += (i + 1) as Index;
		}

		// Add vertices
		for v in 0..n + 1 {
			let pt = bezier2(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2]);
			vp[v] = pen.to_vertex(pt, v);
		}
	}
	fn draw_bezier3(&mut self, pen: &Pen<S>, pts: &[Point2; 4]) {
		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(pen.segments, 2) as usize;
		let (vp, ip) = self.draw_primitive::<S>(Primitive::Lines, n + 1, n);
		debug_assert_eq!(vp.len(), n + 1);
		debug_assert_eq!(ip.len(), n * 2);

		// Add indices
		for i in 0..n {
			ip[i * 2] += i as Index;
			ip[i * 2 + 1] += (i + 1) as Index;
		}

		// Add vertices
		for v in 0..n + 1 {
			let pt = bezier3(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2], pts[3]);
			vp[v] = pen.to_vertex(pt, v);
		}
	}
	fn draw_cspline(&mut self, pen: &Pen<S>, pts: &[Point2], tension: f32) {
		// Degenerate cspline
		if pts.len() < 2 {
			return;
		}

		// On first iteration, incoming velocity is zero
		let mut u = Point2::default();
		let mut v;
		let tension = (1.0 - tension) * 0.5;

		for i in 0..pts.len() - 1 {
			// Calculate the end point velocity
			v = if i == pts.len() - 2 {
				// On last iteration, outgoing velocity is zero
				Point2::default()
			}
			else {
				(pts[i + 2] - pts[i]) * tension
			};
			let curve = [
				pts[i],
				pts[i] + u * (1.0 / 3.0),
				pts[i + 1] - v * (1.0 / 3.0),
				pts[i + 1],
			];
			self.draw_bezier3(pen, &curve);
			u = v;
		}
	}
}
