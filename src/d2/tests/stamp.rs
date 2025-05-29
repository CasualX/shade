use super::*;

#[test]
fn stamp_rect() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let stamp = Stamp {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let rc = Bounds2::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	cbuf.stamp_rect(&stamp, &rc);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn stamp_quad() {
	let mut cbuf = CommandBuffer::<MockVertex, MockUniform>::new();
	let stamp = Stamp {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let pos = Transform2::translate(Vec2::new(10.0, 20.0));
	cbuf.stamp_quad(&stamp, &pos);
	assert_eq!(cbuf.commands.len(), 1);
	assert_eq!(cbuf.vertices.len(), 4);
	assert_eq!(cbuf.indices, &[0, 1, 2, 0, 2, 3]);
}
