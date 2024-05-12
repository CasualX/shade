use super::*;

#[inline]
pub fn bezier2(x: f32, p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>) -> Point2<f32> {
	let s = x;
	let t = 1.0 - s;
	let term1 = p1 * (s * s);
	let term2 = p2 * (s * t + s * t);
	let term3 = p3 * (t * t);
	term1 + term2 + term3
}

#[inline]
pub fn bezier3(x: f32, p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>, p4: Point2<f32>) -> Point2<f32> {
	let s = x;
	let t = 1.0 - s;
	let term1 = p1 * (s * s * s);
	let term2 = p2 * (s * s * t * 3.0);
	let term3 = p3 * (s * t * t * 3.0);
	let term4 = p4 * (t * t * t);
	term1 + term2 + term3 + term4
}
