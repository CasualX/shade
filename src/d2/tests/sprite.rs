use super::*;

#[test]
fn sprite_rect() {
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
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
	let mut buf = DrawBuilder::<MockVertex, MockUniform>::new();
	let stamp = Sprite {
		bottom_left: (),
		top_left: (),
		top_right: (),
		bottom_right: (),
	};
	let pos = Transform2::translation(Vec2(10.0, 20.0));
	buf.sprite_quad(&stamp, &pos);
	assert_eq!(buf.commands.len(), 1);
	assert_eq!(buf.vertices.len(), 4);
	assert_eq!(buf.indices, &[0, 1, 2, 0, 2, 3]);
}

#[test]
fn sprite_transform_matches_atlas_corner_transforms() {
	let stamp = Sprite {
		bottom_left: "bottom_left",
		top_left: "top_left",
		top_right: "top_right",
		bottom_right: "bottom_right",
	};

	let cases = [
		(atlas::Transform::None, ["bottom_left", "top_left", "top_right", "bottom_right"]),
		(atlas::Transform::Rotate90, ["bottom_right", "bottom_left", "top_left", "top_right"]),
		(atlas::Transform::Rotate180, ["top_right", "bottom_right", "bottom_left", "top_left"]),
		(atlas::Transform::Rotate270, ["top_left", "top_right", "bottom_right", "bottom_left"]),
		(atlas::Transform::FlipX, ["bottom_right", "top_right", "top_left", "bottom_left"]),
		(atlas::Transform::FlipY, ["top_left", "bottom_left", "bottom_right", "top_right"]),
		(atlas::Transform::FlipSlash, ["bottom_left", "bottom_right", "top_right", "top_left"]),
		(atlas::Transform::FlipBackslash, ["top_right", "top_left", "bottom_left", "bottom_right"]),
	];

	for (transform, expected) in cases {
		let transformed = stamp.transform(transform);
		assert_eq!(
			[
				transformed.bottom_left,
				transformed.top_left,
				transformed.top_right,
				transformed.bottom_right,
			],
			expected,
			"{transform:?}",
		);
	}
}
