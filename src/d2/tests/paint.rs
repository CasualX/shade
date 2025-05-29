use super::*;

#[test]
fn fill_rect() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	cbuf.fill_rect(&paint, &rc);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_edge_rect() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	let thickness = 10.0;
	cbuf.fill_edge_rect(&paint, &rc, thickness);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 8);
	assert_eq!(cbuf.indices, &[0, 5, 4, 0, 1, 5, 1, 6, 5, 1, 2, 6, 2, 7, 6, 2, 3, 7, 3, 4, 7, 3, 0, 4]);
}

#[test]
fn fill_convex() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(0.0, 100.0),
	];
	cbuf.fill_convex(&paint, &pts);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 3, 1, 2, 3]);
}

#[test]
fn fill_polygon() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
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
	cbuf.fill_polygon(&paint, &pts, &triangles);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_quad() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let bottom_left = Point2::new(0.0, 0.0);
	let top_left = Point2::new(0.0, 100.0);
	let top_right = Point2::new(100.0, 100.0);
	let bottom_right = Point2::new(100.0, 0.0);
	cbuf.fill_quad(&paint, &bottom_left, &top_left, &top_right, &bottom_right);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn fill_ellipse() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	cbuf.fill_ellipse(&paint, &rc, 32);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 33);
	assert_eq!(cbuf.indices.len(), 32 * 3);
}

#[test]
fn fill_pie() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	cbuf.fill_pie(&paint, &rc, Rad(0.0), Rad(90.0), 4);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 6);
	assert_eq!(cbuf.indices.len(), 4 * 3);
}

#[test]
fn fill_ring() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	let thickness = 10.0;
	cbuf.fill_ring(&paint, &rc, thickness, 32);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 64);
	assert_eq!(cbuf.indices.len(), 64 * 3);
}

#[test]
fn fill_bezier2() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let paint = Paint {
		template: (),
	};
	let pivot = Point2::new(0.0, 0.0);
	let p0 = Point2::new(0.0, 0.0);
	let p1 = Point2::new(100.0, 0.0);
	let p2 = Point2::new(100.0, 100.0);
	cbuf.fill_bezier2(&paint, &pivot, &[p0, p1, p2], 32);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 34);
	assert_eq!(cbuf.indices.len(), 32 * 3);
}
