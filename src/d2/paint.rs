use super::*;


/// Paint bucket fills shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint<T> {
	/// Vertex template.
	pub template: T,
}

impl<V: TVertex, U: TUniform> CommandBuffer<V, U> {
	/// Fills a rectangle.
	#[inline(never)]
	pub fn fill_rect<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>) {
		let vertices = [
			paint.template.to_vertex(rc.bottom_left(), 0),
			paint.template.to_vertex(rc.top_left(), 1),
			paint.template.to_vertex(rc.top_right(), 2),
			paint.template.to_vertex(rc.bottom_right(), 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}

	/// Fills a rectangle with an outline.
	#[inline(never)]
	pub fn fill_edge_rect<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>, thickness: f32) {
		let vertices = [
			paint.template.to_vertex(rc.top_left(), 1),
			paint.template.to_vertex(rc.top_right(), 2),
			paint.template.to_vertex(rc.bottom_right(), 3),
			paint.template.to_vertex(rc.bottom_left(), 0),
			paint.template.to_vertex(rc.top_left() + (thickness, thickness), 5),
			paint.template.to_vertex(rc.top_right() + (-thickness, thickness), 6),
			paint.template.to_vertex(rc.bottom_right() + (-thickness, -thickness), 7),
			paint.template.to_vertex(rc.bottom_left() + (thickness, -thickness), 4)
		];
		let mut cv = self.begin(PrimType::Triangles, 8, 8);
		cv.add_indices(&[
			0, 5, 4, 0, 1, 5,
			1, 6, 5, 1, 2, 6,
			2, 7, 6, 2, 3, 7,
			3, 4, 7, 3, 0, 4,
		]);
		cv.add_vertices(&vertices);
	}

	/// Fills a rounded rectangle.
	#[inline(never)]
	pub fn fill_round_rect<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>, sx: f32, sy: f32, _segments: i32) {
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

	/// Fills a convex shape.
	#[inline(never)]
	pub fn fill_convex<T: ToVertex<V>>(&mut self, paint: &Paint<T>, pts: &[Point2<f32>]) {
		// Degenerate convex shape
		if pts.len() < 3 {
			return;
		}
		// n vertices, n - 2 primitives, (n - 2) * 3 indices
		let n = pts.len() - 2;
		let mut cv = self.begin(PrimType::Triangles, pts.len(), n);
		// Add indices
		for i in 0..n as u32 {
			cv.add_index3(i, i + 1, n as u32 + 1);
		}
		// Add vertices
		for v in 0..pts.len() {
			cv.add_vertex(paint.template.to_vertex(pts[v], v));
		}
	}

	/// Fills a polygon shape.
	#[inline(never)]
	pub fn fill_polygon<T: ToVertex<V>>(&mut self, paint: &Paint<T>, pts: &[Point2<f32>], triangles: &[(u32, u32, u32)]) {
		let mut cv = self.begin(PrimType::Triangles, pts.len(), triangles.len());
		// Add indices
		for i in 0..triangles.len() {
			// Must be indices into the points slice
			let (p1, p2, p3) = triangles[i];
			let _ = pts[p1 as usize];
			let _ = pts[p2 as usize];
			let _ = pts[p3 as usize];
			cv.add_index3(p1, p2, p3);
		}
		// Add vertices
		for v in 0..pts.len() {
			cv.add_vertex(paint.template.to_vertex(pts[v], v));
		}
	}

	/// Fills an arbitrary quad.
	#[inline(never)]
	pub fn fill_quad<T: ToVertex<V>>(&mut self, paint: &Paint<T>, bottom_left: &Point2<f32>, top_left: &Point2<f32>, top_right: &Point2<f32>, bottom_right: &Point2<f32>) {
		let vertices = [
			paint.template.to_vertex(*bottom_left, 0),
			paint.template.to_vertex(*top_left, 1),
			paint.template.to_vertex(*top_right, 2),
			paint.template.to_vertex(*bottom_right, 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}

	/// Fills an ellipse.
	#[inline(never)]
	pub fn fill_ellipse<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>, segments: i32) {
		// n + 1 vertices, n primitives, n * 3 indices
		let n = cmp::max(3, segments) as usize;
		let mut cv = self.begin(PrimType::Triangles, n + 1, n);

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add indices
		for i in 1..n {
			cv.add_index3(0, i as u32, i as u32 + 1);
		}
		cv.add_index3(0, n as u32 - 1, 1);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		cv.add_vertex(paint.template.to_vertex(center, 0));
		for i in 0..n {
			let pos = pt * radius + center;
			cv.add_vertex(paint.template.to_vertex(pos, i + 1));
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}

	/// Fills a pie slice.
	#[inline(never)]
	pub fn fill_pie<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>, start: Rad<f32>, sweep: Rad<f32>, segments: i32) {
		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(2, segments) as usize;
		let mut cv = self.begin(PrimType::Triangles, n + 2, n);

		// Add indices
		for i in 1..n + 1 {
			cv.add_index3(0, i as u32, i as u32 + 1);
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
		cv.add_vertex(paint.template.to_vertex(center, 0));
		for i in 0..n + 1 {
			let pos = pt * radius + center;
			cv.add_vertex(paint.template.to_vertex(pos, i + 1));
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}

	/// Fills a ring.
	#[inline(never)]
	pub fn fill_ring<T: ToVertex<V>>(&mut self, paint: &Paint<T>, rc: &Rect<f32>, thickness: f32, segments: i32) {
		// n * 2 vertices, n * 2 primitives, n * 6 indices
		let n = cmp::max(3, segments) as usize;
		let mut cv = self.begin(PrimType::Triangles, n * 2, n * 2);

		// Add indices
		for i in 0..n - 1 {
			let v = i * 2;
			cv.add_index3(v as u32, (v + 1) as u32, (v + 2) as u32);
			cv.add_index3((v + 1) as u32, (v + 2) as u32, (v + 3) as u32);
		}
		{
			let v = (n - 1) * 2;
			cv.add_index3(v as u32, (v + 1) as u32, 0);
			cv.add_index3((v + 1) as u32, 0, 1);
		}

		// Precompute trigs
		let (s, c) = (Rad::turn() / (n as i32 as f32)).sin_cos();
		let radius = rc.size() * 0.5;
		let width = radius - Point2::dup(thickness);
		let center = rc.top_left() + radius;
		let mut pt = Point2(1.0, 0.0);

		// Add vertices
		// http://slabode.exofire.net/circle_draw.shtml
		for i in 0..n {
			let pos = pt * radius + center;
			cv.add_vertex(paint.template.to_vertex(pos, i * 2));
			let pos = pt * width + center;
			cv.add_vertex(paint.template.to_vertex(pos, i * 2 + 1));
			// Apply rotation matrix
			let x = pt.x;
			pt.x = c * x - s * pt.y;
			pt.y = s * x + c * pt.y;
		}
	}

	pub fn fill_bezier2<T: ToVertex<V>>(&mut self, paint: &Paint<T>, pivot: &Point2<f32>, pts: &[Point2<f32>; 3], segments: i32) {
		// n + 2 vertices, n primitives, n * 3 indices
		let n = cmp::max(2, segments) as usize;
		let mut cv = self.begin(PrimType::Triangles, n + 2, n);

		// Add indices
		for i in 1..n + 1 {
			cv.add_index3(0, i as u32, i as u32 + 1);
		}

		// Add vertices
		cv.add_vertex(paint.template.to_vertex(*pivot, 0));
		for v in 1..n + 2 {
			let pt = curve::bezier2(v as i32 as f32 / n as i32 as f32, pts[0], pts[1], pts[2]);
			cv.add_vertex(paint.template.to_vertex(pt, v));
		}
	}
}
