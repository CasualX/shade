use super::*;

/// Decoded image data.
#[derive(Clone, Debug)]
pub enum DecodedImage {
	/// An RGBA image.
	RGBA(ImageRGBA),
	/// An RGB image.
	RGB(ImageRGB),
	/// A grayscale image.
	Grey(Image<u8>),
	/// A grayscale + alpha image.
	GreyAlpha(Image<[u8; 2]>),
	/// An indexed color image with a palette.
	Indexed {
		image: Image<u8>,
		palette: Vec<[u8; 3]>,
	}
}

impl From<ImageRGBA> for DecodedImage {
	#[inline]
	fn from(image: ImageRGBA) -> Self {
		DecodedImage::RGBA(image)
	}
}

impl From<ImageRGB> for DecodedImage {
	#[inline]
	fn from(image: ImageRGB) -> Self {
		DecodedImage::RGB(image)
	}
}

impl From<Image<u8>> for DecodedImage {
	#[inline]
	fn from(image: Image<u8>) -> Self {
		DecodedImage::Grey(image)
	}
}

impl From<Image<[u8; 2]>> for DecodedImage {
	#[inline]
	fn from(image: Image<[u8; 2]>) -> Self {
		DecodedImage::GreyAlpha(image)
	}
}

impl DecodedImage {
	/// Returns the image as RGBA if it is an RGBA image.
	#[inline]
	pub fn rgba(self) -> Option<ImageRGBA> {
		match self {
			DecodedImage::RGBA(image) => Some(image),
			_ => None,
		}
	}
	/// Returns the image as RGB if it is an RGB image.
	#[inline]
	pub fn rgb(self) -> Option<ImageRGB> {
		match self {
			DecodedImage::RGB(image) => Some(image),
			_ => None,
		}
	}
	/// Returns the image as grayscale if it is a grayscale image.
	#[inline]
	pub fn grey(self) -> Option<Image<u8>> {
		match self {
			DecodedImage::Grey(image) => Some(image),
			_ => None,
		}
	}
	/// Returns the image as grayscale + alpha if it is a grayscale + alpha image.
	#[inline]
	pub fn grey_alpha(self) -> Option<Image<[u8; 2]>> {
		match self {
			DecodedImage::GreyAlpha(image) => Some(image),
			_ => None,
		}
	}
	/// Returns the image as indexed color if it is an indexed color image.
	#[inline]
	pub fn indexed(self) -> Option<(Image<u8>, Vec<[u8; 3]>)> {
		match self {
			DecodedImage::Indexed { image, palette } => Some((image, palette)),
			_ => None,
		}
	}
}

impl DecodedImage {
	/// Returns the width of the image.
	#[inline]
	pub fn width(&self) -> i32 {
		match self {
			DecodedImage::RGBA(image) => image.width,
			DecodedImage::RGB(image) => image.width,
			DecodedImage::Grey(image) => image.width,
			DecodedImage::GreyAlpha(image) => image.width,
			DecodedImage::Indexed { image, .. } => image.width,
		}
	}
	/// Returns the height of the image.
	#[inline]
	pub fn height(&self) -> i32 {
		match self {
			DecodedImage::RGBA(image) => image.height,
			DecodedImage::RGB(image) => image.height,
			DecodedImage::Grey(image) => image.height,
			DecodedImage::GreyAlpha(image) => image.height,
			DecodedImage::Indexed { image, .. } => image.height,
		}
	}
	/// Converts the image to RGBA format.
	pub fn to_rgba(self) -> ImageRGBA {
		match self {
			DecodedImage::RGBA(image) => image,
			DecodedImage::RGB(image) => image.map_colors(|[r, g, b]| [r, g, b, 255]),
			DecodedImage::Grey(image) => image.map_colors(|v| [v, v, v, 255]),
			DecodedImage::GreyAlpha(image) => image.map_colors(|[v, a]| [v, v, v, a]),
			DecodedImage::Indexed { image, palette } => image.map_colors(|index| {
				let [r, g, b] = palette[index as usize];
				[r, g, b, 255]
			}),
		}
	}
	/// Converts the image to RGB format.
	///
	/// The alpha channel is discarded if present.
	pub fn to_rgb(self) -> ImageRGB {
		match self {
			DecodedImage::RGBA(image) => image.map_colors(|[r, g, b, _a]| [r, g, b]),
			DecodedImage::RGB(image) => image,
			DecodedImage::Grey(image) => image.map_colors(|v| [v, v, v]),
			DecodedImage::GreyAlpha(image) => image.map_colors(|[v, _a]| [v, v, v]),
			DecodedImage::Indexed { image, palette } => image.map_colors(|index| palette[index as usize]),
		}
	}
}

impl From<DecodedImage> for ImageRGBA {
	#[inline]
	fn from(img: DecodedImage) -> Self {
		img.to_rgba()
	}
}

impl From<DecodedImage> for ImageRGB {
	#[inline]
	fn from(img: DecodedImage) -> Self {
		img.to_rgb()
	}
}

fn load_file(path: &path::Path) -> Result<DecodedImage, LoadImageError> {
	#![allow(unused_variables)]
	let ext = path.extension().and_then(|s| s.to_str());
	#[cfg(feature = "png")]
	if ext == Some("png") {
		return DecodedImage::load_file_png(path).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if ext == Some("gif") {
		return DecodedImage::load_file_gif(path).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if ext == Some("jpg") || ext == Some("jpeg") {
		return DecodedImage::load_file_jpeg(path).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

fn load_memory(data: &[u8]) -> Result<DecodedImage, LoadImageError> {
	#![allow(unused_variables)]
	#[cfg(feature = "png")]
	if data.starts_with(io::png::PNG_SIGNATURE) {
		return DecodedImage::load_memory_png(data).map_err(Into::into);
	}
	#[cfg(feature = "gif")]
	if data.starts_with(io::gif::GIF_SIGNATURE_87A) || data.starts_with(io::gif::GIF_SIGNATURE_89A) {
		return DecodedImage::load_memory_gif(data).map_err(Into::into);
	}
	#[cfg(feature = "jpeg")]
	if data.starts_with(io::jpeg::JPEG_SIGNATURE) {
		return DecodedImage::load_memory_jpeg(data).map_err(Into::into);
	}
	Err(LoadImageError::UnsupportedFormat)
}

impl DecodedImage {
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
