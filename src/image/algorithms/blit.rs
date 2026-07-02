use super::*;

#[inline]
fn blit<T: Copy>(dest: &mut Image<T>, offset: Point2i, src: &Image<T>) {
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

fn blit_blend<T: Copy, F: Fn(T, T) -> T>(dest: &mut Image<T>, offset: Point2i, src: &Image<T>, src_rect: Recti, blend: F) {
	if src_rect.width <= 0 || src_rect.height <= 0 {
		return;
	}

	let src_xstart = i32::max(0, src_rect.x);
	let src_xend = i32::min(src.width, src_rect.x + src_rect.width);
	let src_ystart = i32::max(0, src_rect.y);
	let src_yend = i32::min(src.height, src_rect.y + src_rect.height);
	if src_xstart >= src_xend || src_ystart >= src_yend {
		return;
	}

	let dst_xstart = offset.x + src_xstart - src_rect.x;
	let dst_xend = dst_xstart + src_xend - src_xstart;
	let dst_ystart = offset.y + src_ystart - src_rect.y;
	let dst_yend = dst_ystart + src_yend - src_ystart;

	let xstart = i32::max(0, dst_xstart);
	let xend = i32::min(dest.width, dst_xend);
	let ystart = i32::max(0, dst_ystart);
	let yend = i32::min(dest.height, dst_yend);
	if xstart >= xend || ystart >= yend {
		return;
	}

	for y in ystart..yend {
		let dst_slice = dest.row_span_mut(y, xstart, xend).unwrap();
		let src_y = src_ystart + y - dst_ystart;
		let src_xstart = src_xstart + xstart - dst_xstart;
		let src_xend = src_xstart + xend - xstart;
		let src_slice = src.row_span(src_y, src_xstart, src_xend).unwrap();
		for (dst, src) in dst_slice.iter_mut().zip(src_slice.iter()) {
			*dst = blend(*dst, *src);
		}
	}
}

fn fill_rect<T: Copy, F: Fn(T, Point2i) -> T>(dest: &mut Image<T>, rect: Recti, f: F) {
	if rect.width <= 0 || rect.height <= 0 {
		return;
	}

	let xstart = i32::max(0, rect.x);
	let xend = i32::min(dest.width, rect.x + rect.width);
	let ystart = i32::max(0, rect.y);
	let yend = i32::min(dest.height, rect.y + rect.height);
	if xstart >= xend || ystart >= yend {
		return;
	}

	for y in ystart..yend {
		let dst_slice = dest.row_span_mut(y, xstart, xend).unwrap();
		for (i, x) in (xstart..xend).enumerate() {
			dst_slice[i] = f(dst_slice[i], Point2i(x - rect.x, y - rect.y));
		}
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

fn blit_edge<T: Copy>(dest: &mut Image<T>, offset: Point2i, src: &Image<T>, gutter: i32) {
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

fn blit_border<T: Copy>(dest: &mut Image<T>, offset: Point2i, src: &Image<T>, gutter: i32, color: T) {
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
			}
			else {
				color
			};
		}
	}
}

fn blit_repeat<T: Copy>(dest: &mut Image<T>, offset: Point2i, src: &Image<T>, gutter: i32) {
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
	pub fn copy_from(&mut self, offset: Point2i, src: &Image<T>) {
		blit(self, offset, src)
	}
	/// Copies pixels from another image.
	///
	/// Apply a blending function to each destination/source pixel pair.
	#[inline]
	pub fn copy_blend<F: Fn(T, T) -> T>(&mut self, offset: Point2i, src: &Image<T>, src_rect: Recti, blend: F) {
		blit_blend(self, offset, src, src_rect, blend)
	}
	/// Fills a rectangle on the image.
	///
	/// The rectangle is filled with pixels generated by the provided function given the current pixel value and the pixel's position relative inside the rect.
	#[inline]
	pub fn fill_rect<F: Fn(T, Point2i) -> T>(&mut self, rect: Recti, f: F) {
		fill_rect(self, rect, f)
	}
	/// Copies pixels from another image.
	///
	/// Adds a gutter around the copied area by extending the edge pixels.
	#[inline]
	pub fn copy_with_gutter(&mut self, offset: Point2i, src: &Image<T>, gutter: i32, mode: BlitGutterMode<T>) {
		match mode {
			BlitGutterMode::Edge => blit_edge(self, offset, src, gutter),
			BlitGutterMode::Border(color) => blit_border(self, offset, src, gutter, color),
			BlitGutterMode::Repeat => blit_repeat(self, offset, src, gutter),
		}
	}
}
