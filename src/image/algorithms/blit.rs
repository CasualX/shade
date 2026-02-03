use super::*;

#[inline]
fn blit<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i) {
	let ystart = i32::max(0, offset.y);
	let yend = i32::min(dest.height, offset.y + src.height);
	assert!(ystart <= yend);

	let xstart = i32::max(0, offset.x);
	let xend = i32::min(dest.width, offset.x + src.width);
	assert!(xstart <= xend);

	for y in ystart..yend {
		let dst_slice = dest.row_span_mut(y, xstart, xend).unwrap();
		let src_slice = src.row_span(y - offset.y, xstart - offset.x, xend - offset.x).unwrap();
		dst_slice.copy_from_slice(src_slice);
	}
}

/// How pixels outside the source rect are generated when blitting with a gutter.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlitGutterMode<T> {
	/// Extend edge pixels outward.
	Edge,
	/// Fill the gutter with a constant color.
	Border(T),
	/// Repeat (wrap) the source image.
	Repeat,
}

fn blit_edge<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i, gutter: i32) {
	let start_x = i32::max(0, offset.x - gutter);
	let start_y = i32::max(0, offset.y - gutter);
	let end_x = i32::min(dest.width, offset.x + src.width + gutter);
	let end_y = i32::min(dest.height, offset.y + src.height + gutter);
	if start_x >= end_x || start_y >= end_y {
		return;
	}

	for y in start_y..end_y {
		// Clamp source row so vertical gutter extends edge rows
		let src_y = i32::clamp(y - offset.y, 0, src.height - 1);
		let dst_row = dest.row_span_mut(y, start_x, end_x).unwrap();
		for (i, x) in (start_x..end_x).enumerate() {
			let src_x = i32::clamp(x - offset.x, 0, src.width - 1);
			dst_row[i] = src.read(src_x, src_y).unwrap();
		}
	}
}

fn blit_border<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i, gutter: i32, color: T) {
	let gutter = i32::max(0, gutter);

	let start_x = i32::max(0, offset.x - gutter);
	let start_y = i32::max(0, offset.y - gutter);
	let end_x = i32::min(dest.width, offset.x + src.width + gutter);
	let end_y = i32::min(dest.height, offset.y + src.height + gutter);
	if start_x >= end_x || start_y >= end_y {
		return;
	}

	let src_x0 = offset.x;
	let src_y0 = offset.y;
	let src_x1 = offset.x + src.width;
	let src_y1 = offset.y + src.height;

	for y in start_y..end_y {
		let dst_row = dest.row_span_mut(y, start_x, end_x).unwrap();
		for (i, x) in (start_x..end_x).enumerate() {
			let in_src = x >= src_x0 && x < src_x1 && y >= src_y0 && y < src_y1;
			dst_row[i] = if in_src {
				src.read(x - offset.x, y - offset.y).unwrap()
			} else {
				color
			};
		}
	}
}

fn blit_repeat<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i, gutter: i32) {
	let gutter = i32::max(0, gutter);

	let start_x = i32::max(0, offset.x - gutter);
	let start_y = i32::max(0, offset.y - gutter);
	let end_x = i32::min(dest.width, offset.x + src.width + gutter);
	let end_y = i32::min(dest.height, offset.y + src.height + gutter);
	if start_x >= end_x || start_y >= end_y {
		return;
	}

	if src.width <= 0 || src.height <= 0 {
		return;
	}

	for y in start_y..end_y {
		let src_y = (y - offset.y).rem_euclid(src.height);
		let dst_row = dest.row_span_mut(y, start_x, end_x).unwrap();
		for (i, x) in (start_x..end_x).enumerate() {
			let src_x = (x - offset.x).rem_euclid(src.width);
			dst_row[i] = src.read(src_x, src_y).unwrap();
		}
	}
}

impl<T: Copy> Image<T> {
	/// Copies pixels from another image.
	#[inline]
	pub fn copy_from(&mut self, src: &Image<T>, offset: Point2i) {
		blit(self, src, offset)
	}
	/// Copies pixels from another image.
	///
	/// Adds a gutter around the copied area by extending the edge pixels.
	#[inline]
	pub fn copy_with_gutter(&mut self, src: &Image<T>, offset: Point2i, gutter: i32, mode: BlitGutterMode<T>) {
		match mode {
			BlitGutterMode::Edge => blit_edge(self, src, offset, gutter),
			BlitGutterMode::Border(color) => blit_border(self, src, offset, gutter, color),
			BlitGutterMode::Repeat => blit_repeat(self, src, offset, gutter),
		}
	}
}
