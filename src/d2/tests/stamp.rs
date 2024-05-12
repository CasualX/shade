use super::*;

#[test]
fn stamp_rect() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let stamp = Stamp {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let rc = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
	canvas.stamp_rect(&stamp, &rc);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn stamp_quad() {
	let mut canvas = Canvas::<MockVertex, MockUniform>::new();
	let stamp = Stamp {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let pos = Transform2::translate(Vec2::new(10.0, 20.0));
	canvas.stamp_quad(&stamp, &pos);
	assert_eq!(canvas.commands.len(), 1);
	assert_eq!(canvas.vertices.len(), 4);
	assert_eq!(canvas.indices, &[0, 1, 2, 0, 2, 3]);
}
