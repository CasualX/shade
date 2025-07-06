use super::*;

#[test]
fn draw_line() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	buf.draw_line(&pen, Point2(0.0, 0.0), Point2(100.0, 100.0));
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 2);
	assert_eq!(buf.indices, &[0, 1]);
}

#[test]
fn draw_lines() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let pts = [
		Point2(0.0, 0.0),
		Point2(100.0, 100.0),
		Point2(200.0, 200.0),
	];
	let lines = [(0, 1), (1, 2)];
	buf.draw_lines(&pen, &pts, &lines);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 3);
	assert_eq!(buf.indices, &[0, 1, 1, 2]);
}

#[test]
fn draw_line_rect() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let rc = Bounds2(Point2(0.0, 0.0), Point2(100.0, 100.0));
	buf.draw_line_rect(&pen, &rc);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 4);
	assert_eq!(buf.indices, &[0, 1, 1, 2, 2, 3, 3, 0]);
}

#[test]
fn draw_poly_line() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	let pts = [
		Point2(0.0, 0.0),
		Point2(100.0, 100.0),
		Point2(200.0, 200.0),
	];
	buf.draw_poly_line(&pen, &pts, true);
	buf.draw_poly_line(&pen, &pts, false);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 6);
	assert_eq!(buf.indices, &[
		0, 1, 1, 2, 2, 0,
		3, 4, 4, 5,
	]);
}

#[test]
fn draw_ellipse() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	buf.draw_ellipse(&pen, &Bounds2(Point2(0.0, 0.0), Point2(100.0, 100.0)), 128);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 128);
	assert_eq!(buf.indices.len(), 128 * 2);
}

#[test]
fn draw_arc() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	buf.draw_arc(&pen, &Bounds2(Point2(0.0, 0.0), Point2(100.0, 100.0)), Rad(0.0), Rad(1.0), 128);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 129);
	assert_eq!(buf.indices.len(), 128 * 2);
}

#[test]
fn draw_bezier2() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	buf.draw_bezier2(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
	], 5);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 6);
	assert_eq!(buf.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}

#[test]
fn draw_bezier3() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
	};
	buf.draw_bezier3(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
		Point2(3.25, 0.0),
	], 5);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 6);
	assert_eq!(buf.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}
