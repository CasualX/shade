
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
	/// <svg width="400" height="200" viewBox="-2 0 8 5" xmlns="http://www.w3.org/2000/svg">
	///   <rect fill="none" stroke="green" stroke-width="0.025" x="-1" y="1" width="8" height="3.5" />
	///   <polygon fill="none" stroke="black" stroke-width="0.025" points="1,1 7,2 3,4.5 -1,4" />
	/// </svg>
	///
	/// The example below visualizes the input points (the black polygon) and its resulting bounding box (the green rectangle).
	///
	/// ```
	/// use shade::d2::{Polygon, Point2, Rect};
	///
	/// let pts = vec![
	/// 	Point2(1.0, 1.0),
	/// 	Point2(7.0, 2.0),
	/// 	Point2(3.0, 4.5),
	/// 	Point2(-1.0, 4.0),
	/// ];
	///
	/// let result = Rect(Point2(-1.0, 1.0), Point2(7.0, 4.5));
	/// assert_eq!(pts.bounds(), result);
	/// ```
	fn bounds(&self) -> Rect;
	fn ball(&self) -> (Point2, f32);
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
	///
	/// Returns indices into the polygon which represent the convex hull.
	///
	/// # Examples
	///
	/// <svg width="400" height="200" viewBox="-2 -3 8 8" xmlns="http://www.w3.org/2000/svg">
	///   <polygon fill="none" stroke="green" stroke-width="0.05" points="-1,-1 1,-2 5,-1 5,2 4,4 2,3 -1,1" />
	///   <circle cx="-1.0" cy="-1.0" r="0.1" />
	///   <circle cx="-1.0" cy="1.0" r="0.1" />
	///   <circle cx="1.0" cy="2.0" r="0.1" />
	///   <circle cx="2.0" cy="3.0" r="0.1" />
	///   <circle cx="3.0" cy="2.0" r="0.1" />
	///   <circle cx="4.0" cy="4.0" r="0.1" />
	///   <circle cx="5.0" cy="2.0" r="0.1" />
	///   <circle cx="5.0" cy="-1.0" r="0.1" />
	///   <circle cx="3.0" cy="-1.0" r="0.1" />
	///   <circle cx="1.0" cy="-2.0" r="0.1" />
	/// </svg>
	///
	/// The example below visualizes the input points and its convex hull (the green polygon).
	///
	/// ```
	/// let pts = &[
	/// 	Point2(-1.0, -1.0),
	/// 	Point2(-1.0, 1.0),
	/// 	Point2(1.0, 2.0),
	/// 	Point2(2.0, 3.0),
	/// 	Point2(3.0, 2.0),
	/// 	Point2(4.0, 4.0),
	/// 	Point2(5.0, 2.0),
	/// 	Point2(5.0, -1.0),
	/// 	Point2(3.0, -1.0),
	/// 	Point2(1.0, -2.0),
	/// ];
	/// let hull = pts.convex_hull();
	/// assert_eq!(hull, &[0, 10, 8, 6, 5, 3, 1]);
	/// ```
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
	fn ball(&self) -> (Point2, f32) {
		let pts = self.as_ref();
		if pts.is_empty() {
			return (Point2::default(), 0.0);
		}

		let mut mins = pts[0];
		let mut maxs = pts[0];
		let (mut p_xmin, mut p_xmax, mut p_ymin, mut p_ymax) = (&pts[0], &pts[0], &pts[0], &pts[0]);
		for p in &pts[1..] {
			if p.x < mins.x {
				mins.x = p.x;
				p_xmin = p;
			}
			else if p.x > maxs.x {
				maxs.x = p.x;
				p_xmax = p;
			}
			if p.y < mins.y {
				mins.y = p.y;
				p_ymin = p;
			}
			else if p.y > maxs.y {
				maxs.y = p.y;
				p_ymax = p;
			}
		}

		let mut p_c;
		let mut radius2;
		let mut radius;

		let p_dx = *p_xmax - *p_xmin;
		let p_dy = *p_ymax - *p_ymin;
		if p_dx.len_sqr() > p_dy.len_sqr() {
			p_c = *p_xmin + p_dx * 0.5;
			radius2 = (*p_xmax - p_c).len_sqr();
		}
		else {
			p_c = *p_ymin + p_dy * 0.5;
			radius2 = (*p_ymax - p_c).len_sqr();
		}
		radius = radius2.sqrt();

		for &p in pts {
			let d_p = p - p_c;
			let dist2 = d_p.len_sqr();
			if dist2 <= radius2 {
				continue;
			}
			let dist = dist2.sqrt();
			radius = (radius + dist) * 0.5;
			radius2 = radius * radius;
			p_c = p_c + d_p * ((dist - radius) / dist);
		}

		(p_c, radius)
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

		let pts = self.as_ref();
		if pts.len() < 2 {
			let mut result = Vec::new();
			if pts.len() == 1 {
				result.push(0);
			}
			return result;
		}

		// Cross product between (OA) and (OB)
		let cross = |o: Index, a: Index, b: Index| -> f32 {
			let o = pts[o as usize];
			let a = pts[a as usize];
			let b = pts[b as usize];
			(a - o).cross(b - o)
		};

		// Allocate small polygons on the stack for sorting
		const STACK_SIZE: usize = 64;
		let mut array: [Index; STACK_SIZE];
		let mut owned: Vec<Index>;
		let sorted = if pts.len() > STACK_SIZE {
			owned = (0..pts.len() as Index).collect();
			&mut owned
		}
		else {
			array = unsafe { ::std::mem::uninitialized() };
			for i in 0..pts.len() {
				array[i] = i as Index;
			}
			&mut array[..pts.len()]
		};
		// TODO: Unstable sort when available
		sorted.sort_by(|&a, &b| {
			use ::std::cmp::Ordering;
			let a = pts[a as usize];
			let b = pts[b as usize];
			let order = if a.x == b.x { a.y < b.y } else { a.x < b.x };
			if order { Ordering::Less }
			else { Ordering::Greater }
		});

		// Allocate worst-case space
		let mut result = Vec::with_capacity(pts.len() + 1);
		unsafe { result.set_len(pts.len() + 1); }

		let mut k = 0;
		for i in 0..pts.len() {
			while k >= 2 && cross(result[k - 2], result[k - 1], sorted[i]) <= 0.0 {
				k -= 1;
			}
			result[k] = sorted[i];
			k += 1;
		}

		let t = k + 1;
		for i in (0..pts.len() - 1).rev() {
			while k >= t && cross(result[k - 2], result[k - 1], sorted[i]) <= 0.0 {
				k -= 1;
			}
			result[k] = sorted[i];
			k += 1;
		}

		unsafe { result.set_len(k - 1); }
		return result;
	}
	fn triangulate(&self) -> Vec<(Index, Index, Index)> {
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

	#[test]
	fn convex_hull() {
		let pts = &[
			Point2(-1.0, -1.0),
			Point2(-1.0, 1.0),
			Point2(1.0, 2.0),
			Point2(2.0, 3.0),
			Point2(3.0, 2.0),
			Point2(4.0, 4.0),
			Point2(5.0, 2.0),
			Point2(5.0, -1.0),
			Point2(5.0, -2.0),
			Point2(3.0, -1.0),
			Point2(1.0, -2.0),
		];
		let hull = pts.convex_hull();
		assert_eq!(hull, &[0, 10, 8, 6, 5, 3, 1]);
	}
}
