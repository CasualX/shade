
use super::super::{IPen, Pen, Point2, Rect, Rad, ColorV};
use super::MockCanvas;
use {Primitive, TShader, TUniform};

#[derive(Copy, Clone, Debug, Default)]
struct PenShader;
impl TUniform for PenShader {
	fn uniform_uid() -> u32 { 12334 }
}
impl TShader for PenShader {
	type Vertex = ColorV;
	type Uniform = ();
	fn shader_uid() -> u32 { 21847 }
}

#[test]
fn draw_line() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		shader: PenShader,
		..Pen::default()
	};
	cv.draw_line(&pen, Point2(-0.5, 4.0), Point2(-3.25, 8.125));

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 1);
	assert_eq!(cv.verts.len(), 2);
	assert_eq!(cv.indices, &[1, 2]);
}

#[test]
fn draw_line_rect() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		shader: PenShader,
		..Pen::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.draw_line_rect(&pen, &rc);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 4);
	assert_eq!(cv.verts.len(), 4);
	assert_eq!(cv.indices, &[1, 2, 2, 3, 3, 4, 4, 1]);
}

#[test]
fn draw_poly_line() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		shader: PenShader,
		..Pen::default()
	};
	let pts = [
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
	];
	cv.draw_poly_line(&pen, &pts, false);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 2);
	assert_eq!(cv.verts.len(), 3);
	assert_eq!(cv.indices, &[1, 2, 2, 3]);

	cv.draw_poly_line(&pen, &pts, true);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 3);
	assert_eq!(cv.verts.len(), 6);
	assert_eq!(cv.indices, &[5, 6, 6, 7, 7, 5]);
}

#[test]
fn draw_ellipse() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		segments: 8,
		shader: PenShader,
		..Pen::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.0), Point2(1.0, 1.0));
	cv.draw_ellipse(&pen, &rc);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 8);
	assert_eq!(cv.verts.len(), 8);
	assert_eq!(cv.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 1]);
}

#[test]
fn draw_arc() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		segments: 3,
		shader: PenShader,
		..Pen::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.0), Point2(1.0, 1.0));
	cv.draw_arc(&pen, &rc, Rad::eight(), Rad::quarter());

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 3);
	assert_eq!(cv.verts.len(), 4);
	assert_eq!(cv.indices, &[1, 2, 2, 3, 3, 4]);
}

#[test]
fn draw_bezier2() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		segments: 5,
		shader: PenShader,
		..Pen::default()
	};
	let pts = [
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
	];
	cv.draw_bezier2(&pen, &pts);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 5);
	assert_eq!(cv.verts.len(), 6);
	assert_eq!(cv.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
}

#[test]
fn draw_bezier3() {
	let mut cv = MockCanvas::<ColorV>::default();
	let pen = Pen {
		segments: 5,
		shader: PenShader,
		..Pen::default()
	};
	let pts = [
		Point2(1.0, 2.0),
		Point2(-2.0, 4.5),
		Point2(0.5, 1.0),
		Point2(3.25, 0.0),
	];
	cv.draw_bezier3(&pen, &pts);

	assert_eq!(cv.prim, Primitive::Lines);
	assert_eq!(cv.nprims, 5);
	assert_eq!(cv.verts.len(), 6);
	assert_eq!(cv.indices, &[1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
}
