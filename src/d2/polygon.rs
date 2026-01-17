use super::*;

pub fn triangulate(positions: &[Point2f]) -> Vec<Index3> {
	const EPS: f32 = 1.0e-6;

	let n = positions.len();
	if n < 3 {
		return Vec::new();
	}
	assert!(n <= u32::MAX as usize, "triangulate: too many vertices for u16 indices");

	fn cross(a: Vec2f, b: Vec2f) -> f32 {
		a.cross(b)
	}

	fn signed_area(pts: &[Vec2f]) -> f32 {
		let mut sum = 0.0f32;
		for i in 0..pts.len() {
			let a = pts[i];
			let b = pts[(i + 1) % pts.len()];
			sum += a.x * b.y - b.x * a.y;
		}
		sum * 0.5
	}

	fn is_convex_ccw(a: Vec2f, b: Vec2f, c: Vec2f, eps: f32) -> bool {
		cross(b - a, c - a) > eps
	}

	fn point_in_triangle_ccw(p: Vec2f, a: Vec2f, b: Vec2f, c: Vec2f, eps: f32) -> bool {
		// Accept points on the edge as inside (prevents degenerate ears)
		cross(b - a, p - a) >= -eps && cross(c - b, p - b) >= -eps && cross(a - c, p - c) >= -eps
	}

	// Ensure counter-clockwise winding for consistent convexity / inside tests.
	let ccw = signed_area(&positions) >= 0.0;
	let mut remaining: Vec<usize> = if ccw {
		(0..n).collect()
	} else {
		(0..n).rev().collect()
	};

	let mut triangles: Vec<Index3> = Vec::with_capacity(n.saturating_sub(2));
	let mut guard = 0usize;
	let guard_limit = n.saturating_mul(n).saturating_add(8);

	while remaining.len() > 3 {
		if guard > guard_limit {
			break;
		}
		guard += 1;

		let len = remaining.len();
		let mut ear_found = false;

		for i in 0..len {
			let prev = remaining[(i + len - 1) % len];
			let curr = remaining[i];
			let next = remaining[(i + 1) % len];

			let a = positions[prev];
			let b = positions[curr];
			let c = positions[next];

			if !is_convex_ccw(a, b, c, EPS) {
				continue;
			}

			// Reject ears that contain any other vertex.
			let mut contains_other = false;
			for &idx in &remaining {
				if idx == prev || idx == curr || idx == next {
					continue;
				}
				if point_in_triangle_ccw(positions[idx], a, b, c, EPS) {
					contains_other = true;
					break;
				}
			}
			if contains_other {
				continue;
			}

			triangles.push(Index3 {
				p1: prev as u32,
				p2: curr as u32,
				p3: next as u32,
			});
			remaining.remove(i);
			ear_found = true;
			break;
		}

		if !ear_found {
			// Fallback: drop a nearly-collinear vertex to make progress.
			let len = remaining.len();
			let mut dropped = false;
			for i in 0..len {
				let prev = remaining[(i + len - 1) % len];
				let curr = remaining[i];
				let next = remaining[(i + 1) % len];
				let a = positions[prev];
				let b = positions[curr];
				let c = positions[next];
				if cross(b - a, c - a).abs() <= EPS {
					remaining.remove(i);
					dropped = true;
					break;
				}
			}
			if !dropped {
				break;
			}
		}
	}

	if remaining.len() == 3 {
		triangles.push(Index3 {
			p1: remaining[0] as u32,
			p2: remaining[1] as u32,
			p3: remaining[2] as u32,
		});
	}

	triangles
}
