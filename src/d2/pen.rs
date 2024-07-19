use super::*;

/// Pencil draws lines.
#[derive(Clone, Debug, PartialEq)]
pub struct Pen<T> {
	/// Vertex template.
	pub template: T,
}

impl<V: TVertex, U: TUniform> CommandBuffer<V, U> {
	/// Draws a line from `a` to `b`.
	#[inline(never)]
	pub fn draw_line<T: ToVertex<V>>(&mut self, pen: &Pen<T>, a: Point2<f32>, b: Point2<f32>) {
		let vertices = [
			pen.template.to_vertex(a, 0),
			pen.template.to_vertex(b, 1),
		];
		let mut cv = self.begin(PrimType::Lines, 2, 1);
		cv.add_index2(0, 1);
		cv.add_vertices(&vertices);
	}

	/// Draws lines.
	#[inline(never)]
	pub fn draw_lines<T: ToVertex<V>>(&mut self, pen: &Pen<T>, pts: &[Point2<f32>], lines: &[(u32, u32)]) {
		let mut cv = self.begin(PrimType::Lines, pts.len(), lines.len());
		for i in 0..lines.len() {
			cv.add_index2(lines[i].0, lines[i].1);
		}
		for i in 0..pts.len() {
			cv.add_vertex(pen.template.to_vertex(pts[i], i));
		}
	}

	/// Draws a rectangle with lines.
	#[inline(never)]
	pub fn draw_line_rect<T: ToVertex<V>>(&mut self, pen: &Pen<T>, rc: &Rect<f32>) {
		let vertices = [
			pen.template.to_vertex(rc.bottom_left(), 0),
			pen.template.to_vertex(rc.top_left(), 1),
			pen.template.to_vertex(rc.top_right(), 2),
			pen.template.to_vertex(rc.bottom_right(), 3),
		];
		let mut cv = self.begin(PrimType::Lines, 4, 4);
		cv.add_indices(&[0, 1, 1, 2, 2, 3, 3, 0]);
		cv.add_vertices(&vertices);
	}

	/// Draws a rounded rectangle with lines.
	#[inline(never)]
	pub fn draw_round_rect<T: ToVertex<V>>(&mut self, pen: &Pen<T>, rc: &Rect<f32>, sx: f32, sy: f32, _segments: i32) {
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

	/// Draws an arrow.
	pub fn draw_arrow<T: ToVertex<V>>(&mut self, pen: &Pen<T>, start: Point2<f32>, end: Point2<f32>, head: f32) {
		let pth = (end - start).resize(head);
		let pta = (end - pth) + pth.ccw() * 0.5;
		let ptb = (end - pth) + pth.cw() * 0.5;
		let vertices = [
			pen.template.to_vertex(start, 0),
			pen.template.to_vertex(end, 1),
			pen.template.to_vertex(pta, 2),
			pen.template.to_vertex(ptb, 3),
		];
		let mut cv = self.begin(PrimType::Lines, 4, 3);
		cv.add_indices(&[0, 1, 2, 1, 3, 1]);
		cv.add_vertices(&vertices);
	}

	/// Draws connected lines.
	#[inline(never)]
	pub fn draw_poly_line<T: ToVertex<V>>(&mut self, pen: &Pen<T>, pts: &[Point2<f32>], close: bool) {
		// Degenerate polyline
		if pts.len() < 2 {
			return;
		}
		// open: n vertices, n - 1 primitives, (n - 1) * 2 indices
		// close: n vertices, n primitives, n * 2 indices
		let n = pts.len() - (!close) as usize;
		let mut cv = self.begin(PrimType::Lines, pts.len(), n);
		// Add indices
		for i in 0..pts.len() - 1 {
			cv.add_index2(i as u32, (i + 1) as u32);
		}
		if close {
			cv.add_index2((pts.len() - 1) as u32, 0);
		}
		// Add vertices
		for v in 0..pts.len() {
			cv.add_vertex(pen.template.to_vertex(pts[v], v));
		}
	}

	/// Draws an ellipse.
	#[inline(never)]
	pub fn draw_ellipse<T: ToVertex<V>>(&mut self, pen: &Pen<T>, rc: &Rect<f32>, segments: i32) {
		// n vertices, n primitives, n * 2 indices
		let n = cmp::max(3, segments) as usize;
		let mut cv = self.begin(PrimType::Lines, n, n);

		// Add indices
		for i in 0..n - 1 {
			cv.add_index2(i as u32, (i + 1) as u32);
		}
		cv.add_index2((n - 1) as u32, 0);

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for v in 0..n {
			cv.add_vertex(pen.template.to_vertex(center + pt * radius, v));
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}

	/// Draws an arc.
	#[inline(never)]
	pub fn draw_arc<T: ToVertex<V>>(&mut self, pen: &Pen<T>, rc: &Rect<f32>, start: Rad<f32>, sweep: Rad<f32>, segments: i32) {
		if sweep <= -Rad::turn() || sweep >= Rad::turn() {
			return self.draw_ellipse(pen, rc, segments);
		}

		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(2, segments) as usize;
		let mut cv = self.begin(PrimType::Lines, n + 1, n);

		// Add indices
		for i in 0..n {
			cv.add_index2(i as u32, (i + 1) as u32);
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
		for v in 0..n + 1 {
			cv.add_vertex(pen.template.to_vertex(center + pt * radius, v));
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}

	#[inline(never)]
	pub fn draw_bezier2<T: ToVertex<V>>(&mut self, pen: &Pen<T>, pts: &[Point2<f32>; 3], segments: i32) {
		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(2, segments) as usize;
		let mut cv = self.begin(PrimType::Lines, n + 1, n);

		// Add indices
		for i in 0..n {
			cv.add_index2(i as u32, (i + 1) as u32);
		}

		// Add vertices
		for v in 0..n + 1 {
			let pt = curve::bezier2(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2]);
			cv.add_vertex(pen.template.to_vertex(pt, v));
		}
	}

	#[inline(never)]
	pub fn draw_bezier3<T: ToVertex<V>>(&mut self, pen: &Pen<T>, pts: &[Point2<f32>; 4], segments: i32) {
		// n + 1 vertices, n primitives, n * 2 indices
		let n = cmp::max(2, segments) as usize;
		let mut cv = self.begin(PrimType::Lines, n + 1, n);

		// Add indices
		for i in 0..n {
			cv.add_index2(i as u32, (i + 1) as u32);
		}

		// Add vertices
		for v in 0..n + 1 {
			let pt = curve::bezier3(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2], pts[3]);
			cv.add_vertex(pen.template.to_vertex(pt, v));
		}
	}

	#[inline(never)]
	pub fn draw_cspline<T: ToVertex<V>>(&mut self, pen: &Pen<T>, pts: &[Point2<f32>], tension: f32, segments: i32) {
		// Degenerate cspline
		if pts.len() < 2 {
			return;
		}

		// On first iteration, incoming velocity is zero
		let mut u = Point2::ZERO;
		let mut v;
		let tension = (1.0 - tension) * 0.5;

		for i in 0..pts.len() - 1 {
			// Calculate the end point velocity
			v = if i == pts.len() - 2 {
				// On last iteration, outgoing velocity is zero
				Point2::ZERO
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
			self.draw_bezier3(pen, &curve, segments);
			u = v;
		}
	}
}
