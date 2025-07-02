use super::*;

#[test]
fn sprite_rect() {
	let mut buf = DrawBuffer::<MockVertex, MockUniform>::new();
	let stamp = Sprite {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let rc = Bounds2(Point2(0.0, 0.0), Point2(100.0, 100.0));
	buf.sprite_rect(&stamp, &rc);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 4);
	assert_eq!(buf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn sprite_quad() {
	let mut buf = DrawBuffer::<MockVertex, MockUniform>::new();
	let stamp = Sprite {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let pos = Transform2::translate(Vec2(10.0, 20.0));
	buf.sprite_quad(&stamp, &pos);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 4);
	assert_eq!(buf.indices, &[0, 1, 2, 0, 2, 3]);
}
