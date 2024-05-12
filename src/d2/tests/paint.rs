use super::*;

#[test]
fn fill_rect() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 1,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	canvas.fill_rect(&paint, &rc);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_edge_rect() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 1,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	let thickness = 10.0;
	canvas.fill_edge_rect(&paint, &rc, thickness);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 8);
	assert_eq!(canvas.indices, &[0, 5, 4, 0, 1, 5, 1, 6, 5, 1, 2, 6, 2, 7, 6, 2, 3, 7, 3, 4, 7, 3, 0, 4]);
}

#[test]
fn fill_convex() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 1,
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(0.0, 100.0),
	];
	canvas.fill_convex(&paint, &pts);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 3, 1, 2, 3]);
}

#[test]
fn fill_polygon() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 1,
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(0.0, 100.0),
	];
	let triangles = [
		(0, 1, 2),
		(0, 2, 3),
	];
	canvas.fill_polygon(&paint, &pts, &triangles);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_quad() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 1,
	};
	let bottom_left = Point2::new(0.0, 0.0);
	let top_left = Point2::new(0.0, 100.0);
	let top_right = Point2::new(100.0, 100.0);
	let bottom_right = Point2::new(100.0, 0.0);
	canvas.fill_quad(&paint, &bottom_left, &top_left, &top_right, &bottom_right);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_ellipse() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 32,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	canvas.fill_ellipse(&paint, &rc);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 33);
	assert_eq!(canvas.indices.len(), 32 * 3);
}

#[test]
fn fill_pie() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 4,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	canvas.fill_pie(&paint, &rc, cvmath::Rad(0.0), cvmath::Rad(90.0));
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 6);
	assert_eq!(canvas.indices.len(), 4 * 3);
}

#[test]
fn fill_ring() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 32,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	let thickness = 10.0;
	canvas.fill_ring(&paint, &rc, thickness);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 64);
	assert_eq!(canvas.indices.len(), 64 * 3);
}

#[test]
fn fill_bezier2() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
		segments: 32,
	};
	let pivot = Point2::new(0.0, 0.0);
	let p0 = Point2::new(0.0, 0.0);
	let p1 = Point2::new(100.0, 0.0);
	let p2 = Point2::new(100.0, 100.0);
	canvas.fill_bezier2(&paint, &pivot, &[p0, p1, p2]);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 34);
	assert_eq!(canvas.indices.len(), 32 * 3);
}
