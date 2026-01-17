use super::*;

#[test]
fn triangulate_convex_quad() {
	let verts = [
		Vec2f(0.0, 0.0),
		Vec2f(1.0, 0.0),
		Vec2f(1.0, 1.0),
		Vec2f(0.0, 1.0),
	];
	let tris = crate::d2::polygon::triangulate(&verts);
	assert_eq!(tris.len(), 2);
	for t in tris {
		assert!(t.p1 < 4 && t.p2 < 4 && t.p3 < 4);
		assert!(t.p1 != t.p2 && t.p2 != t.p3 && t.p3 != t.p1);
	}
}

#[test]
fn triangulate_concave_polygon() {
	// Simple concave "arrow" shape
	let verts = [
		Vec2f(0.0, 0.0),
		Vec2f(2.0, 0.0),
		Vec2f(2.0, 1.0),
		Vec2f(1.0, 0.5), // inward dent
		Vec2f(0.0, 1.0),
	];
	let tris = crate::d2::polygon::triangulate(&verts);
	assert_eq!(tris.len(), 3);
	for t in tris {
		assert!(t.p1 < 5 && t.p2 < 5 && t.p3 < 5);
	}
}
