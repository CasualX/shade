
use super::{Point2, Rect};

pub trait Polygon {
	/// Returns an iterator over the edges of the polygon.
	fn edges(&self) -> Edges;
	/// Calculates the bounding box of the polygon.
	fn bounds(&self) -> Rect;
	/// Calculates the signed area of the polygon.
	///
	/// The winding of a polygon is defined to be counter-clockwise if its signed area is positive and clockwise if its signed area is negative.
	fn area(&self) -> f32;
	/// Calculates the crossing number of a point with the polygon.
	fn crossing_number(&self, pt: Point2) -> u32;
	/// Calculates the winding number of a point with the polygon.
	fn winding_number(&self, pt: Point2) -> i32;
	/// Calculates the enclosing convex hull of the polygon.
	fn convex_hull(&self) -> Vec<u32>;
	/// Triangulates the polygon.
	fn triangulate(&self) -> Vec<u32>;
}

//----------------------------------------------------------------

impl<T: ?Sized + AsRef<[Point2]>> Polygon for T {
	fn edges(&self) -> Edges {
		static DUMMY: Point2 = Point2 { x: 0.0, y: 0.0 };
		let pts = self.as_ref();
		if pts.len() > 0 {
			Edges { pj: &DUMMY, iter: pts.iter() }
		}
		else {
			Edges { pj: &pts[pts.len() - 1], iter: pts.iter() }
		}
	}
	fn bounds(&self) -> Rect {
		let pts = self.as_ref();
		let (mins, maxs) = if pts.len() == 0 {
			Default::default()
		}
		else {
			pts.iter().fold((pts[0], pts[0]), |acc, p| (acc.0.min(*p), acc.1.max(*p)))
		};
		Rect::new(mins, maxs)
	}
	fn area(&self) -> f32 {
		let pts = self.as_ref();
		let mut acc = 0.0;
		if pts.len() > 0 {
			let mut pj = &pts[pts.len() - 1];
			for pi in pts {
				acc += pj.x * pi.y - pi.x * pj.y;
				pj = pi;
			}
		}
		return acc * 0.5;
	}
	fn crossing_number(&self, pt: Point2) -> u32 {
		let mut cn = 0;
		let pts = self.as_ref();
		if pts.len() > 0 {
			let mut pj = &pts[pts.len() - 1];
			for pi in pts {
				if (pi.y > pt.y) != (pj.y > pt.y) {
					if pt.x - pi.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) {
						cn += 1;
					}
				}
				pj = pi;
			}
		}
		return cn;
	}
	fn winding_number(&self, pt: Point2) -> i32 {
		let mut wn = 0;
		let pts = self.as_ref();
		if pts.len() > 0 {
			#[inline(always)]
			fn is_left(v0: &Point2, v1: &Point2, pt: Point2) -> f32 {
				(v1.x - v0.x) * (pt.y - v0.y) - (pt.x - v0.x) * (v1.y - v0.y)
			}
			let mut pj = &pts[pts.len() - 1];
			for pi in pts {
				if pj.y <= pt.y {
					if pi.y > pt.y {
						let is_left = is_left(pj, pi, pt);
						if is_left > 0.0 {
							wn += 1;
						}
					}
				}
				else if pi.y <= pt.y {
					let is_left = is_left(pj, pi, pt);
					if is_left < 0.0 {
						wn -= 1;
					}
				}
				pj = pi;
			}
		}
		return wn;
	}
	fn convex_hull(&self) -> Vec<u32> {
		let pts = self.as_ref();
		// Create a vector with indices
		let mut sorted = Vec::with_capacity(pts.len());
		unsafe { sorted.set_len(pts.len()); }
		for i in 0..pts.len() { sorted[i] = i as u32; }
		// Sort by X coordinate
		sorted.sort_by(|&v1, &v2| {
			use ::std::cmp::Ordering::*;
			let v1 = &pts[v1 as usize];
			let v2 = &pts[v2 as usize];
			if v1.x < v2.x { Less }
			else { Greater }
		});


		unimplemented!()
	}
	fn triangulate(&self) -> Vec<u32> {
		unimplemented!()
	}
}

//----------------------------------------------------------------

use ::std::slice;

pub struct Edges<'a> {
	iter: slice::Iter<'a, Point2>,
	pj: &'a Point2,
}
impl<'a> Iterator for Edges<'a> {
	type Item = (&'a Point2, &'a Point2);
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|pi| {
			let pj = self.pj;
			self.pj = pi;
			(pj, pi)
		})
	}
}

