use super::*;

#[cfg(test)]
mod tests;

/// In memory image buffer.
#[derive(Clone, Debug)]
pub struct Image<T> {
	pub width: i32,
	pub height: i32,
	pub data: Vec<T>,
}

impl<T: Copy> Image<T> {
	/// Returns the number of pixels in the image.
	#[inline]
	pub fn len(&self) -> usize {
		(self.width as i64 * self.height as i64) as usize
	}
	/// Creates a new image filled with the specified pixel value.
	#[inline]
	pub fn new(width: i32, height: i32, fill: T) -> Image<T> {
		let len = (width as i64 * height as i64) as usize;
		Image {
			width,
			height,
			data: vec![fill; len],
		}
	}
	/// Creates a new image from raw pixel data.
	#[inline]
	pub fn from_raw(width: i32, height: i32, data: Vec<T>) -> Image<T> {
		let len = (width as i64 * height as i64) as usize;
		assert_eq!(data.len(), len);
		Image { width, height, data }
	}
	/// Maps the colors of the image to another pixel type.
	#[inline]
	pub fn map_colors<U, F: FnMut(T) -> U>(self, f: F) -> Image<U> {
		// Optimized to avoid double allocation if possible
		let data = self.data.into_iter().map(f).collect();
		Image {
			width: self.width,
			height: self.height,
			data,
		}
	}
	/// Transmutes to another pixel type.
	#[inline]
	pub fn transmute<U: dataview::Pod>(self) -> Image<U> where T: dataview::Pod {
		assert_eq!(mem::size_of::<T>(), mem::size_of::<U>());
		assert_eq!(mem::align_of::<T>(), mem::align_of::<U>());
		Image {
			width: self.width,
			height: self.height,
			data: unsafe { mem::transmute(self.data) },
		}
	}
	/// Returns the buffer as a byte slice.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] where T: dataview::Pod {
		dataview::bytes(&*self.data)
	}
	/// Returns a row of the image.
	#[inline]
	pub fn row(&self, y: i32) -> Option<&[T]> {
		self.row_span(y, 0, self.width)
	}
	/// Returns a row of the image.
	#[inline]
	pub fn row_span(&self, y: i32, x_start: i32, x_end: i32) -> Option<&[T]> {
		if y < 0 || y >= self.height {
			return None;
		}
		if !(x_start >= 0 && x_end <= self.width && x_start <= x_end) {
			return None;
		}
		let start = (y as i64 * self.width as i64 + x_start as i64) as usize;
		let end = start + (x_end - x_start) as usize;
		self.data.get(start..end)
	}
	/// Returns a mutable row of the image.
	#[inline]
	pub fn row_mut(&mut self, y: i32) -> Option<&mut [T]> {
		let width = self.width;
		self.row_span_mut(y, 0, width)
	}
	/// Returns a mutable row of the image.
	#[inline]
	pub fn row_span_mut(&mut self, y: i32, x_start: i32, x_end: i32) -> Option<&mut [T]> {
		if y < 0 || y >= self.height {
			return None;
		}
		if !(x_start >= 0 && x_end <= self.width && x_start <= x_end) {
			return None;
		}
		let start = (y as i64 * self.width as i64 + x_start as i64) as usize;
		let end = start + (x_end - x_start) as usize;
		self.data.get_mut(start..end)
	}
	/// Returns a sub-image view of a rectangular region.
	#[inline]
	pub fn sub_image(&self, x: i32, y: i32, width: i32, height: i32) -> Option<Image<T>> {
		if width < 0 || height < 0 {
			return None;
		}
		if x < 0 || y < 0 || x + width > self.width || y + height > self.height {
			return None;
		}
		let mut data = Vec::with_capacity((width as i64 * height as i64) as usize);
		for row in 0..height {
			let src_row = self.row_span(y + row, x, x + width).unwrap();
			data.extend_from_slice(src_row);
		}
		Some(Image { width, height, data })
	}
	/// Gets the index of a pixel in the buffer.
	#[inline]
	pub fn index(&self, x: i32, y: i32) -> Option<usize> {
		if x < 0 || x >= self.width || y < 0 || y >= self.height {
			return None;
		}
		Some((y as i64 * self.width as i64 + x as i64) as usize)
	}
	/// Reads a pixel from the image.
	#[inline]
	pub fn read(&self, x: i32, y: i32) -> Option<T> {
		let index = self.index(x, y)?;
		self.data.get(index).copied()
	}
	/// Writes a pixel to the image.
	#[inline]
	pub fn write(&mut self, x: i32, y: i32, value: T) {
		if let Some(index) = self.index(x, y) {
			if let Some(p) = self.data.get_mut(index) {
				*p = value;
			}
		}
	}
}

fn load_file<T>(path: &path::Path) -> Result<Image<T>, LoadImageError> where Image<T>: From<DecodedImage> {
	#![allow(unused_variables)]
	let ext = path.extension().and_then(|s| s.to_str());
	#[cfg(feature = "png")]
	if ext == Some("png") {
		return Image::load_file_png(path).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if ext == Some("gif") {
		return Image::load_file_gif(path).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if ext == Some("jpg") || ext == Some("jpeg") {
		return Image::load_file_jpeg(path).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

fn load_memory<T>(data: &[u8]) -> Result<Image<T>, LoadImageError> where Image<T>: From<DecodedImage> {
	#![allow(unused_variables)]
	#[cfg(feature = "png")]
	if data.starts_with(io::png::PNG_SIGNATURE) {
		return Image::load_memory_png(data).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if data.starts_with(io::gif::GIF_SIGNATURE_87A) || data.starts_with(io::gif::GIF_SIGNATURE_89A) {
		return Image::load_memory_gif(data).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if data.starts_with(io::jpeg::JPEG_SIGNATURE) {
		return Image::load_memory_jpeg(data).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

impl<T> Image<T> where Image<T>: From<DecodedImage> {
	/// Loads an image from a file.
	#[inline]
	pub fn load_file(path: impl AsRef<path::Path>) -> Result<Self, LoadImageError> {
		load_file(path.as_ref())
	}
	/// Loads an image from memory.
	#[inline]
	pub fn load_memory(data: &[u8]) -> Result<Self, LoadImageError> {
		load_memory(data)
	}
}

pub type ImageRGB = Image<[u8; 3]>;
pub type ImageRGBA = Image<[u8; 4]>;
