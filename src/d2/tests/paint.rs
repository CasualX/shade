
use super::super::{IPaint, Paint, Point2, Rect, Rad, ColorV};
use super::MockCanvas;
use {Primitive, TShader, TUniform};

#[derive(Copy, Clone, Debug, Default)]
struct PaintShader;
impl TUniform for PaintShader {
	fn uid() -> u32 { 1826731 }
}
impl TShader for PaintShader {
	type Vertex = ColorV;
	type Uniform = ();
}

#[test]
fn fill_rect() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		shader: PaintShader,
		..Paint::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.fill_rect(&paint, &rc);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 2);
	assert_eq!(cv.verts[0].pt, rc.top_left());
	assert_eq!(cv.verts[1].pt, rc.top_right());
	assert_eq!(cv.verts[2].pt, rc.bottom_right());
	assert_eq!(cv.verts[3].pt, rc.bottom_left());
	assert_eq!(cv.indices, &[1, 2, 3, 1, 3, 4]);
}

#[test]
fn fill_edge_rect() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		shader: PaintShader,
		..Paint::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.fill_edge_rect(&paint, &rc, 1.0);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 8);
	assert_eq!(cv.verts.len(), 8);
	assert_eq!(cv.indices, &[
		1, 6, 5, 1, 2, 6,
		2, 7, 6, 2, 3, 7,
		3, 8, 7, 3, 4, 8,
		4, 5, 8, 4, 1, 5
	]);
}

#[test]
fn fill_convex() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		shader: PaintShader,
		..Paint::default()
	};
	let pts = [
		Point2(-1.0, -1.0),
		Point2(-1.5, 1.0),
		Point2(0.0, 2.0),
		Point2(1.5, 1.0),
		Point2(1.0, -1.0),
	];
	cv.fill_convex(&paint, &pts);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 3);
	assert_eq!(cv.verts.len(), 5);
	assert_eq!(cv.indices, &[
		1, 2, 5,
		5, 2, 4,
		2, 4, 3,
	]);
}

#[test]
fn fill_quad() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		shader: PaintShader,
		..Paint::default()
	};
	let top_left = Point2(-1.0, 1.0);
	let top_right = Point2(1.0, 1.0);
	let bottom_right = Point2(1.0, -1.0);
	let bottom_left = Point2(-1.0, -1.0);
	cv.fill_quad(&paint, &top_left, &top_right, &bottom_left, &bottom_right);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 2);
	assert_eq!(cv.verts.len(), 4);
	assert_eq!(cv.verts[0].pt, top_left);
	assert_eq!(cv.verts[1].pt, top_right);
	assert_eq!(cv.verts[2].pt, bottom_right);
	assert_eq!(cv.verts[3].pt, bottom_left);
	assert_eq!(cv.indices, &[1, 2, 3, 1, 3, 4]);
}

#[test]
fn fill_ellipse() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		segments: 8,
		shader: PaintShader,
		..Paint::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.fill_ellipse(&paint, &rc);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 8);
	assert_eq!(cv.verts.len(), 9);
	assert_eq!(cv.indices, &[
		1, 2, 3,
		1, 3, 4,
		1, 4, 5,
		1, 5, 6,
		1, 6, 7,
		1, 7, 8,
		1, 8, 9,
		1, 9, 2,
	]);
}

#[test]
fn fill_pie() {
	let mut cv = MockCanvas::<ColorV>::default();
	let paint = Paint {
		segments: 5,
		shader: PaintShader,
		..Paint::default()
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.fill_pie(&paint, &rc, Rad::eight(), Rad::quarter());

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 5);
	assert_eq!(cv.verts.len(), 7);
	assert_eq!(cv.indices, &[
		1, 2, 3,
		1, 3, 4,
		1, 4, 5,
		1, 5, 6,
		1, 6, 7,
	]);
}
