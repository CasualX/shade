use super::*;

/// Animated image.
pub struct AnimatedImage {
	pub width: i32,
	pub height: i32,
	pub frames: Vec<Image<[u8; 4]>>,
	pub delays: Vec<f32>,
	pub repeat: bool,
}

fn load_file(path: &path::Path) -> Result<AnimatedImage, LoadImageError> {
	#![allow(unused_variables)]
	let ext = path.extension().and_then(|s| s.to_str());
	#[cfg(feature = "png")]
	if ext == Some("png") {
		return AnimatedImage::load_file_png(path).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if ext == Some("gif") {
		return AnimatedImage::load_file_gif(path).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if ext == Some("jpg") || ext == Some("jpeg") {
		return AnimatedImage::load_file_jpeg(path).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

fn load_memory(data: &[u8]) -> Result<AnimatedImage, LoadImageError> {
	#![allow(unused_variables)]
	#[cfg(feature = "png")]
	if data.starts_with(io::png::PNG_SIGNATURE) {
		return AnimatedImage::load_memory_png(data).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if data.starts_with(io::gif::GIF_SIGNATURE_87A) || data.starts_with(io::gif::GIF_SIGNATURE_89A) {
		return AnimatedImage::load_memory_gif(data).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if data.starts_with(io::jpeg::JPEG_SIGNATURE) {
		return AnimatedImage::load_memory_jpeg(data).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

impl AnimatedImage {
	/// Loads an animated image from a file.
	#[inline]
	pub fn load_file(path: impl AsRef<path::Path>) -> Result<Self, LoadImageError> {
		load_file(path.as_ref())
	}

	/// Loads an animated image from memory.
	#[inline]
	pub fn load_memory(data: &[u8]) -> Result<Self, LoadImageError> {
		load_memory(data)
	}
}
