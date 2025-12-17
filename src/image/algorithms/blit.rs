use super::*;

#[inline]
pub fn blit<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i) {
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


pub fn blit_gutter<T: Copy>(dest: &mut Image<T>, src: &Image<T>, offset: Point2i, gutter: i32) {
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
	pub fn copy_with_gutter(&mut self, src: &Image<T>, offset: Point2i, gutter: i32) {
		blit_gutter(self, src, offset, gutter)
	}
}
