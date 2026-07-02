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
		dest.copy_from(Point2i(-1, -1), &src);
		// Overlapping top-left 3x3 should be 2, rest 1.
		assert_rect(&dest, 0, 0, 3, 3, 2, 1);
	}

	// Bottom-right copy (5,5)
	{
		let mut dest = dest.clone();
		dest.copy_from(Point2i(5, 5), &src);
		// Overlapping bottom-right 3x3 should be 2, rest 1.
		assert_rect(&dest, 5, 5, 3, 3, 2, 1);
	}

	// Center copy (2,2)
	{
		let mut dest = dest.clone();
		dest.copy_from(Point2i(2, 2), &src);
		// Center 4x4 should be 2, rest 1.
		assert_rect(&dest, 2, 2, 4, 4, 2, 1);
	}
}

#[test]
fn copy_blend_copies_source_subrect() {
	let mut dest = Image::new(5, 4, 0u8);
	let src = Image::from_raw(4, 3, vec![
		1u8, 2u8, 3u8, 4u8,
		5u8, 6u8, 7u8, 8u8,
		9u8, 10u8, 11u8, 12u8,
	]);

	dest.copy_blend(Point2i(1, 1), &src, cvmath::Rect!(1, 0, 3, 2), |_dst, src| src);

	assert_eq!(dest.row(0).unwrap(), &[0, 0, 0, 0, 0]);
	assert_eq!(dest.row(1).unwrap(), &[0, 2, 3, 4, 0]);
	assert_eq!(dest.row(2).unwrap(), &[0, 6, 7, 8, 0]);
	assert_eq!(dest.row(3).unwrap(), &[0, 0, 0, 0, 0]);
}

#[test]
fn copy_blend_clips_subrect_against_source_and_dest() {
	let mut dest = Image::new(4, 3, 0u8);
	let src = Image::from_raw(4, 3, vec![
		1u8, 2u8, 3u8, 4u8,
		5u8, 6u8, 7u8, 8u8,
		9u8, 10u8, 11u8, 12u8,
	]);

	dest.copy_blend(Point2i(-1, -1), &src, cvmath::Rect!(-1, -1, 4, 4), |_dst, src| src + 20);

	assert_eq!(dest.row(0).unwrap(), &[21, 22, 23, 0]);
	assert_eq!(dest.row(1).unwrap(), &[25, 26, 27, 0]);
	assert_eq!(dest.row(2).unwrap(), &[29, 30, 31, 0]);
}

#[test]
fn copy_blend_clips_subrect_against_dest() {
	let mut dest = Image::new(4, 3, 0u8);
	let src = Image::from_raw(4, 3, vec![
		1u8, 2u8, 3u8, 4u8,
		5u8, 6u8, 7u8, 8u8,
		9u8, 10u8, 11u8, 12u8,
	]);

	dest.copy_blend(Point2i(-2, -1), &src, cvmath::Rect!(0, 0, 4, 3), |_dst, src| src);

	assert_eq!(dest.row(0).unwrap(), &[7, 8, 0, 0]);
	assert_eq!(dest.row(1).unwrap(), &[11, 12, 0, 0]);
	assert_eq!(dest.row(2).unwrap(), &[0, 0, 0, 0]);
}

#[test]
fn copy_blend_receives_destination_and_source() {
	let mut dest = Image::new(4, 3, 10u8);
	let src = Image::from_raw(4, 3, vec![
		1u8, 2u8, 3u8, 4u8,
		5u8, 6u8, 7u8, 8u8,
		9u8, 10u8, 11u8, 12u8,
	]);

	dest.copy_blend(Point2i(1, 1), &src, cvmath::Rect!(1, 0, 3, 2), |dst, src| dst + src);

	assert_eq!(dest.row(0).unwrap(), &[10, 10, 10, 10]);
	assert_eq!(dest.row(1).unwrap(), &[10, 12, 13, 14]);
	assert_eq!(dest.row(2).unwrap(), &[10, 16, 17, 18]);
}

#[test]
fn draw_rect_fills_rect_with_relative_position() {
	let mut img = Image::new(5, 4, 1u8);

	img.fill_rect(cvmath::Rect!(1, 1, 3, 2), |dst, pos| dst + pos.x as u8 + (pos.y as u8 * 10));

	assert_eq!(img.row(0).unwrap(), &[1, 1, 1, 1, 1]);
	assert_eq!(img.row(1).unwrap(), &[1, 1, 2, 3, 1]);
	assert_eq!(img.row(2).unwrap(), &[1, 11, 12, 13, 1]);
	assert_eq!(img.row(3).unwrap(), &[1, 1, 1, 1, 1]);
}

#[test]
fn draw_rect_clips_against_destination() {
	let mut img = Image::new(4, 3, 0u8);

	img.fill_rect(cvmath::Rect!(-2, -1, 4, 3), |_dst, pos| (pos.x + pos.y * 10) as u8);

	assert_eq!(img.row(0).unwrap(), &[12, 13, 0, 0]);
	assert_eq!(img.row(1).unwrap(), &[22, 23, 0, 0]);
	assert_eq!(img.row(2).unwrap(), &[0, 0, 0, 0]);
}

#[test]
fn copy_with_gutter_border() {
	// 4x4 dest initialized with 0
	let mut dest = Image::new(4, 4, 0u8);
	// 2x2 src with distinct values
	let src = Image::from_raw(2, 2, vec![1u8, 2u8, 3u8, 4u8]);

	// Place src at (1,1) with 1px gutter filled with 9
	dest.copy_with_gutter(Point2i(1, 1), &src, 1, BlitGutterMode::Border(9));

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

	dest.copy_with_gutter(Point2i(1, 1), &src, 1, BlitGutterMode::Repeat);

	// (0,0) maps to src (-1,-1) => (1,1) => 4
	assert_eq!(dest.read(0, 0).unwrap(), 4);
	// (1,1) maps to src (0,0) => 1
	assert_eq!(dest.read(1, 1).unwrap(), 1);
	// (3,0) maps to src (2,-1) => (0,1) => 3
	assert_eq!(dest.read(3, 0).unwrap(), 3);
	// (0,3) maps to src (-1,2) => (1,0) => 2
	assert_eq!(dest.read(0, 3).unwrap(), 2);
}
