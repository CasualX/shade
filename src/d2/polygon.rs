
use ::{Index};
use super::{Point2, Rect};

pub trait Polygon {
	/// Returns an iterator over the edges of the polygon.
	///
	/// # Examples
	///
	/// Iteration starts by connecting the first and last points.
	///
	/// ```
	/// use shade::d2::{Polygon, Point2};
	///
	/// let pts = vec![Point2(1.0, 1.0), Point2(2.0, 5.0), Point2(4.0, -1.0)];
	///
	/// let mut iter = pts.edges();
	/// assert_eq!(iter.next(), Some((&pts[2], &pts[0])));
	/// assert_eq!(iter.next(), Some((&pts[0], &pts[1])));
	/// assert_eq!(iter.next(), Some((&pts[1], &pts[2])));
	/// assert_eq!(iter.next(), None);
	/// ```
	fn edges(&self) -> Edges;
	/// Builds a new polygon from given indices into the polygon.
	fn clone_indexed(&self, indices: &[Index]) -> Vec<Point2>;
	/// Calculates the bounding box of the polygon.
	///
	/// # Examples
	///
	/// ```
	/// use shade::d2::{Polygon, Point2, Rect};
	///
	/// let pts = vec![Point2(1.0, 1.0), Point2(2.0, 5.0), Point2(4.0, -1.0)];
	///
	/// let result = Rect(Point2(1.0, -1.0), Point2(4.0, 5.0));
	/// assert_eq!(pts.bounds(), result);
	/// ```
	fn bounds(&self) -> Rect;
	/// Calculates the signed area of the polygon.
	///
	/// If the polygon's signed area is positive its winding is said to be counter-clockwise and clockwise if its signed area is negative.
	///
	/// When the polygon is self-intersecting, the area will be counted twice.
	fn signed_area(&self) -> f32;
	/// Calculates the crossing number of a point with the polygon.
	///
	/// The crossing number calculates how many times the edge of the polygon is crossed as a line is drawn from the point to positive infinity over the X axis.
	///
	/// An even result indicates the point is outside the polygon and an odd result indicates the point is inside the polygon.
	fn crossing_number(&self, pt: Point2) -> u32;
	/// Calculates the winding number of a point with the polygon.
	///
	/// The winding number calculates how many times the polygon wraps around the point.
	///
	/// A zero result indicates the point lies outside the polygon.
	fn winding_number(&self, pt: Point2) -> i32;
	/// Calculates the enclosing convex hull of the polygon.
	fn convex_hull(&self) -> Vec<Index>;
	/// Triangulates the polygon.
	fn triangulate(&self) -> Vec<(Index, Index, Index)>;
}

//----------------------------------------------------------------

impl<T: ?Sized + AsRef<[Point2]>> Polygon for T {
	fn edges(&self) -> Edges {
		let pts = self.as_ref();
		let last = unsafe { &*pts.as_ptr().offset(pts.len() as isize - 1) };
		Edges { pj: last, iter: pts.iter() }
	}
	fn clone_indexed(&self, indices: &[Index]) -> Vec<Point2> {
		let pts = self.as_ref();
		indices.iter().map(|&index| pts[index as usize]).collect()
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
	fn signed_area(&self) -> f32 {
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
	fn convex_hull(&self) -> Vec<Index> {
		// Notes:
		// https://en.wikibooks.org/wiki/Algorithm_Implementation/Geometry/Convex_hull/Monotone_chain
		// http://geomalgorithms.com/a12-_hull-3.html
		let mut sorted = self.as_ref().to_owned();
		convex_hull_inplace(&mut sorted)
	}
	fn triangulate(&self) -> Vec<(Index, Index, Index)> {
		unimplemented!()
	}
}

fn convex_hull_inplace(pts: &mut [Point2]) -> Vec<Index> {
	pts.sort_by(|v1, v2| {
		use ::std::cmp::Ordering;
		if v1.x < v2.x { Ordering::Less }
		else if v1.x == v2.x && v1.y < v2.y { Ordering::Less }
		else { Ordering::Greater }
	});

	let mut result = Vec::with_capacity(pts.len());
	unsafe { result.set_len(pts.len()); }

	// Build the lower hull
	let mut k = 0;
	for i in 0..pts.len() {
		while k >= 2 && Point2::cross(pts[k - 1] - pts[k - 2], pts[i] - pts[k - 2]) <= 0.0 {
			k -= 1;
		}
		result[k] = i as Index;
		k += 1;
	}

	// Build the upper hull
	let t = k + 1;
	let mut i = (pts.len() - 2) as isize;
	while i >= 0 {
		while k >= t && Point2::cross(pts[k - 1] - pts[k - 2], pts[i as usize] - pts[k - 2]) <= 0.0 {
			k -= 1;
		}
		result[k] = i as Index;
		k += 1;
		i -= 1;
	}

	unsafe { result.set_len(k - 1); }
	result
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

//----------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn edges_cases() {
		let pts: &[Point2] = &[];
		assert_eq!(pts.edges().next(), None);

		let pts = [Point2(1.0, -1.0)];
		let result = [(&pts[0], &pts[0])];
		assert!(pts.edges().eq(result.iter().cloned()));

		let pts = [Point2(1.0, -1.0), Point2(-2.0, 2.0)];
		let result = [(&pts[1], &pts[0]), (&pts[0], &pts[1])];
		assert!(pts.edges().eq(result.iter().cloned()));
	}
}
