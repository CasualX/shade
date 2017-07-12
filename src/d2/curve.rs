use super::Point;

#[inline]
pub fn bezier2(x: f32, pts: &[Point; 3]) -> Point {
	let s = x;
	let t = 1.0 - s;
	let term1 = pts[0] * (s * s);
	let term2 = pts[1] * (s * t + s * t);
	let term3 = pts[2] * (t * t);
	term1 + term2 + term3
}

#[inline]
pub fn bezier3(x: f32, pts: &[Point; 4]) -> Point {
	let s = x;
	let t = 1.0 - s;
	let term1 = pts[0] * (s * s * s);
	let term2 = pts[1] * (s * s * t * 3.0);
	let term3 = pts[2] * (s * t * t * 3.0);
	let term4 = pts[3] * (t * t * t);
	term1 + term2 + term3 + term4
}

/*
pub fn bezier(x: f32, pts: &mut [Point]) {
	let s = x;
	let t = 1.0 - s;
	let n = pts.len();
	for i in 0..n - 1 {
		pts[i] = pts[i] * s + pts[i + 1] * t;
	}
	bezier(x, &mut pts[..n - 1]);
}
*/
