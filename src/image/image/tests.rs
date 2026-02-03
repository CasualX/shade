use super::*;

#[track_caller]
fn assert_rect(img: &Image<u8>, x0: i32, y0: i32, w: i32, h: i32, inside: u8, outside: u8) {
	for y in 0..img.height {
		for x in 0..img.width {
			let in_rect = x >= x0 && x < x0 + w && y >= y0 && y < y0 + h;
			let expected = if in_rect { inside } else { outside };
			let actual = img.read(x, y).unwrap();
			assert_eq!(actual, expected, "mismatch at ({}, {})", x, y);
		}
	}
}

#[test]
fn copy_from_simple() {
	// 8x8 destination initialized with 1 (background color).
	let dest = Image::new(8, 8, 1u8);
	// 4x4 source initialized with 2 (patch color).
	let src = Image::new(4, 4, 2u8);

	// Top-left copy (-1,-1)
	{
		let mut dest = dest.clone();
		dest.copy_from(&src, Point2i(-1, -1));
		// Overlapping top-left 3x3 should be 2, rest 1.
		assert_rect(&dest, 0, 0, 3, 3, 2, 1);
	}

	// Bottom-right copy (5,5)
	{
		let mut dest = dest.clone();
		dest.copy_from(&src, Point2i(5, 5));
		// Overlapping bottom-right 3x3 should be 2, rest 1.
		assert_rect(&dest, 5, 5, 3, 3, 2, 1);
	}

	// Center copy (2,2)
	{
		let mut dest = dest.clone();
		dest.copy_from(&src, Point2i(2, 2));
		// Center 4x4 should be 2, rest 1.
		assert_rect(&dest, 2, 2, 4, 4, 2, 1);
	}
}

#[test]
fn copy_with_gutter_border() {
	// 4x4 dest initialized with 0
	let mut dest = Image::new(4, 4, 0u8);
	// 2x2 src with distinct values
	let src = Image::from_raw(2, 2, vec![1u8, 2u8, 3u8, 4u8]);

	// Place src at (1,1) with 1px gutter filled with 9
	dest.copy_with_gutter(&src, Point2i(1, 1), 1, BlitGutterMode::Border(9));

	assert_eq!(dest.read(0, 0).unwrap(), 9);
	assert_eq!(dest.read(3, 3).unwrap(), 9);

	// Copied center should match src
	assert_eq!(dest.read(1, 1).unwrap(), 1);
	assert_eq!(dest.read(2, 1).unwrap(), 2);
	assert_eq!(dest.read(1, 2).unwrap(), 3);
	assert_eq!(dest.read(2, 2).unwrap(), 4);
}

#[test]
fn copy_with_gutter_repeat() {
	let mut dest = Image::new(4, 4, 0u8);
	let src = Image::from_raw(2, 2, vec![1u8, 2u8, 3u8, 4u8]);

	dest.copy_with_gutter(&src, Point2i(1, 1), 1, BlitGutterMode::Repeat);

	// (0,0) maps to src (-1,-1) => (1,1) => 4
	assert_eq!(dest.read(0, 0).unwrap(), 4);
	// (1,1) maps to src (0,0) => 1
	assert_eq!(dest.read(1, 1).unwrap(), 1);
	// (3,0) maps to src (2,-1) => (0,1) => 3
	assert_eq!(dest.read(3, 0).unwrap(), 3);
	// (0,3) maps to src (-1,2) => (1,0) => 2
	assert_eq!(dest.read(0, 3).unwrap(), 2);
}
