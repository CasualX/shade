
use super::super::{IStamp, Stamp, Point2, Vec2, Rect, TexV};
use super::MockCanvas;
use {Primitive, TShader, TUniform};

#[derive(Copy, Clone, Debug, Default)]
struct StampShader;
impl TUniform for StampShader {
	fn uid() -> u32 { 1826731 }
}
impl TShader for StampShader {
	type Vertex = TexV;
	type Uniform = ();
}

#[test]
fn stamp_rect() {
	let mut cv = MockCanvas::<TexV>::default();
	let stamp = Stamp {
		uv: Rect(Point2(0.25, 0.25), Point2(0.75, 0.75)),
		shader: StampShader,
	};
	let rc = Rect::new(Point2(-1.0, -1.5), Point2(1.0, 1.5));
	cv.stamp_rect(&stamp, &rc);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 2);
	assert_eq!(cv.verts, [
		TexV { pt: rc.top_left(), uv: stamp.uv.top_left() },
		TexV { pt: rc.top_right(), uv: stamp.uv.top_right() },
		TexV { pt: rc.bottom_right(), uv: stamp.uv.bottom_right() },
		TexV { pt: rc.bottom_left(), uv: stamp.uv.bottom_left() },
	]);
	assert_eq!(cv.indices, &[1, 2, 3, 1, 3, 4]);
}

#[test]
fn stamp_quad() {
	let mut cv = MockCanvas::<TexV>::default();
	let stamp = Stamp {
		uv: Rect(Point2(0.25, 0.25), Point2(0.75, 0.75)),
		shader: StampShader,
	};
	let origin = Point2(1.0, 1.0);
	let x = Vec2(2.0, 0.25);
	let y = Vec2(0.25, 2.0);
	cv.stamp_quad(&stamp, &origin, &x, &y);

	assert_eq!(cv.prim, Primitive::Triangles);
	assert_eq!(cv.nprims, 2);
	assert_eq!(cv.verts, &[
		TexV { pt: origin, uv: stamp.uv.top_left() },
		TexV { pt: origin + x, uv: stamp.uv.top_right() },
		TexV { pt: origin + x + y, uv: stamp.uv.bottom_right() },
		TexV { pt: origin + y, uv: stamp.uv.bottom_left() },
	]);
	assert_eq!(cv.indices, &[1, 2, 3, 1, 3, 4]);
}
