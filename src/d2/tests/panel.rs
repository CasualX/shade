use super::*;

#[test]
fn panel_rect() {
	let mut buf = TexturedBuffer::new();
	let color = Vec4(255, 255, 255, 255);
	let template = TexturedTemplate { uv: Vec2::ZERO, color };
	let uvs = Panel {
		x: [0.0, 0.25, 0.75, 1.0],
		y: [0.0, 0.2, 0.6, 1.0],
	};
	let pos = Panel {
		x: [0.0, 20.0, 80.0, 100.0],
		y: [0.0, 30.0, 90.0, 100.0],
	};

	buf.panel_rect(&template, &uvs, &pos);

	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 16);
	assert_eq!(buf.vertices[0].pos, Point2(0.0, 0.0));
	assert_eq!(buf.vertices[0].uv, Vec2(0.0, 0.0));
	assert_eq!(buf.vertices[5].pos, Point2(20.0, 30.0));
	assert_eq!(buf.vertices[5].uv, Vec2(0.25, 0.2));
	assert_eq!(buf.vertices[10].pos, Point2(80.0, 90.0));
	assert_eq!(buf.vertices[10].uv, Vec2(0.75, 0.6));
	assert_eq!(buf.vertices[15].pos, Point2(100.0, 100.0));
	assert_eq!(buf.vertices[15].uv, Vec2(1.0, 1.0));
}
