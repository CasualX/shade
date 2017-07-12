
use super::super::{IPen, Pen, Point, Rect, Rad, ColorV};
use super::MockShader;
use ::{Primitive};

#[test]
fn draw_line() {
	let mut shader = MockShader::<ColorV>::default();
	let pen = Pen::default();
	shader.draw_line(&pen, Point::new(-0.5, 4.0), Point::new(-3.25, 8.125));

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 1);
	assert_eq!(shader.verts.len(), 2);
	assert_eq!(shader.indices, &[1, 2]);
}

#[test]
fn draw_line_rect() {
	let mut shader = MockShader::<ColorV>::default();
	let pen = Pen::default();
	let rc = Rect::new(Point::new(-1.0, -1.5), Point::new(1.0, 1.5));
	shader.draw_line_rect(&pen, &rc);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 4);
	assert_eq!(shader.verts.len(), 4);
	assert_eq!(shader.indices, &[1, 2, 2, 3, 3, 4, 4, 1]);
}

#[test]
fn draw_poly_line() {
	let mut shader = MockShader::<ColorV>::default();
	let pen = Pen::default();
	let pts = [
		Point::new(1.0, 2.0),
		Point::new(-2.0, 4.5),
		Point::new(0.5, 1.0),
	];
	shader.draw_poly_line(&pen, &pts, false);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 2);
	assert_eq!(shader.verts.len(), 3);
	assert_eq!(shader.indices, &[1, 2, 2, 3]);

	shader.draw_poly_line(&pen, &pts, true);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 3);
	assert_eq!(shader.verts.len(), 3);
	assert_eq!(shader.indices, &[5, 6, 6, 7, 7, 5]);
}

#[test]
fn draw_ellipse() {
	let mut shader = MockShader::<ColorV>::default();
	let mut pen = Pen::default();
	pen.segments = 7;
	let rc = Rect::new(Point::new(-1.0, -1.0), Point::new(1.0, 1.0));
	shader.draw_ellipse(&pen, &rc);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 8);
	assert_eq!(shader.verts.len(), 8);
	assert_eq!(shader.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 1]);
}

#[test]
fn draw_arc() {
	let mut shader = MockShader::<ColorV>::default();
	let mut pen = Pen::default();
	pen.segments = 3;
	let rc = Rect::new(Point::new(-1.0, -1.0), Point::new(1.0, 1.0));
	shader.draw_arc(&pen, &rc, Rad::eight(), Rad::quarter());

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 3);
	assert_eq!(shader.verts.len(), 4);
	assert_eq!(shader.indices, &[1, 2, 2, 3, 3, 4]);
}

#[test]
fn draw_bezier2() {
	let mut shader = MockShader::<ColorV>::default();
	let mut pen = Pen::default();
	pen.segments = 5;
	let pts = [
		Point::new(1.0, 2.0),
		Point::new(-2.0, 4.5),
		Point::new(0.5, 1.0),
	];
	shader.draw_bezier2(&pen, &pts);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 5);
	assert_eq!(shader.verts.len(), 6);
	assert_eq!(shader.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
}

#[test]
fn draw_bezier3() {
	let mut shader = MockShader::<ColorV>::default();
	let mut pen = Pen::default();
	pen.segments = 5;
	let pts = [
		Point::new(1.0, 2.0),
		Point::new(-2.0, 4.5),
		Point::new(0.5, 1.0),
		Point::new(3.25, 0.0),
	];
	shader.draw_bezier3(&pen, &pts);

	assert_eq!(shader.prim, Primitive::Lines);
	assert_eq!(shader.nprims, 5);
	assert_eq!(shader.verts.len(), 6);
	assert_eq!(shader.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
}
