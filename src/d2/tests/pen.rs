use super::*;

#[test]
fn draw_line() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	cbuf.draw_line(&pen, Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 2);
	assert_eq!(cbuf.indices, &[0, 1]);
}

#[test]
fn draw_lines() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(200.0, 200.0),
	];
	let lines = [(0, 1), (1, 2)];
	cbuf.draw_lines(&pen, &pts, &lines);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 3);
	assert_eq!(cbuf.indices, &[0, 1, 1, 2]);
}

#[test]
fn draw_line_rect() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	cbuf.draw_line_rect(&pen, &rc);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 1, 2, 2, 3, 3, 0]);
}

#[test]
fn draw_poly_line() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(200.0, 200.0),
	];
	cbuf.draw_poly_line(&pen, &pts, true);
	cbuf.draw_poly_line(&pen, &pts, false);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 6);
	assert_eq!(cbuf.indices, &[
		0, 1, 1, 2, 2, 0,
		3, 4, 4, 5,
	]);
}

#[test]
fn draw_ellipse() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	cbuf.draw_ellipse(&pen, &Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)), 128);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 128);
	assert_eq!(cbuf.indices.len(), 128 * 2);
}

#[test]
fn draw_arc() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	cbuf.draw_arc(&pen, &Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)), Rad(0.0), Rad(1.0), 128);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 129);
	assert_eq!(cbuf.indices.len(), 128 * 2);
}

#[test]
fn draw_bezier2() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	cbuf.draw_bezier2(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
	], 5);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 6);
	assert_eq!(cbuf.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}

#[test]
fn draw_bezier3() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	cbuf.draw_bezier3(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
		Point2(3.25, 0.0),
	], 5);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 6);
	assert_eq!(cbuf.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}
