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
