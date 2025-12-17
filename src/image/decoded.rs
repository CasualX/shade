use super::*;

/// Decoded image data.
pub enum DecodedImage {
	/// An RGBA image.
	RGBA(Image<[u8; 4]>),
	/// An RGB image.
	RGB(Image<[u8; 3]>),
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

impl From<Image<[u8; 4]>> for DecodedImage {
	#[inline]
	fn from(image: Image<[u8; 4]>) -> Self {
		DecodedImage::RGBA(image)
	}
}

impl From<Image<[u8; 3]>> for DecodedImage {
	#[inline]
	fn from(image: Image<[u8; 3]>) -> Self {
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
	pub fn rgba(self) -> Option<Image<[u8; 4]>> {
		match self {
			DecodedImage::RGBA(image) => Some(image),
			_ => None,
		}
	}
	/// Returns the image as RGB if it is an RGB image.
	#[inline]
	pub fn rgb(self) -> Option<Image<[u8; 3]>> {
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
	pub fn to_rgba(self) -> Image<[u8; 4]> {
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
	pub fn to_rgb(self) -> Image<[u8; 3]> {
		match self {
			DecodedImage::RGBA(image) => image.map_colors(|[r, g, b, _a]| [r, g, b]),
			DecodedImage::RGB(image) => image,
			DecodedImage::Grey(image) => image.map_colors(|v| [v, v, v]),
			DecodedImage::GreyAlpha(image) => image.map_colors(|[v, _a]| [v, v, v]),
			DecodedImage::Indexed { image, palette } => image.map_colors(|index| palette[index as usize]),
		}
	}
}

impl From<DecodedImage> for Image<[u8; 4]> {
	#[inline]
	fn from(img: DecodedImage) -> Self {
		img.to_rgba()
	}
}

impl From<DecodedImage> for Image<[u8; 3]> {
	#[inline]
	fn from(img: DecodedImage) -> Self {
		img.to_rgb()
	}
}
