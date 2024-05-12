use super::*;

#[test]
fn draw_line() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 1,
	};
	canvas.draw_line(&pen, Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 2);
	assert_eq!(canvas.indices, &[0, 1]);
}

#[test]
fn draw_lines() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 1,
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(200.0, 200.0),
	];
	let lines = [(0, 1), (1, 2)];
	canvas.draw_lines(&pen, &pts, &lines);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 3);
	assert_eq!(canvas.indices, &[0, 1, 1, 2]);
}

#[test]
fn draw_line_rect() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 1,
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	canvas.draw_line_rect(&pen, &rc);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 1, 2, 2, 3, 3, 0]);
}

#[test]
fn draw_poly_line() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 1,
	};
	let pts = [
		Point2::new(0.0, 0.0),
		Point2::new(100.0, 100.0),
		Point2::new(200.0, 200.0),
	];
	canvas.draw_poly_line(&pen, &pts, true);
	canvas.draw_poly_line(&pen, &pts, false);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 6);
	assert_eq!(canvas.indices, &[
		0, 1, 1, 2, 2, 0,
		3, 4, 4, 5,
	]);
}

#[test]
fn draw_ellipse() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 128,
	};
	canvas.draw_ellipse(&pen, &Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)));
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 128);
	assert_eq!(canvas.indices.len(), 128 * 2);
}

#[test]
fn draw_arc() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 128,
	};
	canvas.draw_arc(&pen, &Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)), cvmath::Rad(0.0), cvmath::Rad(1.0));
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 129);
	assert_eq!(canvas.indices.len(), 128 * 2);
}

#[test]
fn draw_bezier2() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 5,
	};
	canvas.draw_bezier2(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
	]);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 6);
	assert_eq!(canvas.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}

#[test]
fn draw_bezier3() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let pen = Pen {
		template: (),
		segments: 5,
	};
	canvas.draw_bezier3(&pen, &[
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
		Point2(3.25, 0.0),
	]);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 6);
	assert_eq!(canvas.indices, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
}
