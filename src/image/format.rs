use crate::color::PixelFormat;
use super::*;

/// Load various image types into textures.
pub trait ImageToTexture {
	/// Get the texture info of the image.
	fn info(&self) -> crate::Texture2DInfo;
	/// Get the raw pixel data of the image.
	fn data(&self) -> &[u8];
}

impl ImageToTexture for DecodedImage {
	fn info(&self) -> crate::Texture2DInfo {
		match self {
			DecodedImage::RGBA(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::SRGBA8,
				width: image.width,
				height: image.height,
				props: Default::default(),
			},
			DecodedImage::RGB(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::SRGB8,
				width: image.width,
				height: image.height,
				props: Default::default(),
			},
			DecodedImage::Grey(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::R8,
				width: image.width,
				height: image.height,
				props: Default::default(),
			},
			_ => unimplemented!(),
		}
	}
	fn data(&self) -> &[u8] {
		match self {
			DecodedImage::RGBA(image) => image.as_bytes(),
			DecodedImage::RGB(image) => image.as_bytes(),
			DecodedImage::Grey(image) => image.as_bytes(),
			DecodedImage::GreyAlpha(image) => image.as_bytes(),
			DecodedImage::Indexed { image, .. } => image.as_bytes(),
		}
	}
}

impl ImageToTexture for (&DecodedImage, &crate::TextureProps) {
	fn info(&self) -> crate::Texture2DInfo {
		let (image, &props) = self;
		match image {
			DecodedImage::RGBA(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::SRGBA8,
				width: image.width,
				height: image.height,
				props,
			},
			DecodedImage::RGB(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::SRGB8,
				width: image.width,
				height: image.height,
				props,
			},
			DecodedImage::Grey(image) => crate::Texture2DInfo {
				format: crate::TextureFormat::R8,
				width: image.width,
				height: image.height,
				props,
			},
			_ => unimplemented!(),
		}
	}
	fn data(&self) -> &[u8] {
		match self.0 {
			DecodedImage::RGBA(image) => image.as_bytes(),
			DecodedImage::RGB(image) => image.as_bytes(),
			DecodedImage::Grey(image) => image.as_bytes(),
			DecodedImage::GreyAlpha(image) => image.as_bytes(),
			DecodedImage::Indexed { image, .. } => image.as_bytes(),
		}
	}
}

impl<T: PixelFormat + Copy + dataview::Pod> ImageToTexture for Image<T> {
	#[inline]
	fn info(&self) -> crate::Texture2DInfo {
		crate::Texture2DInfo {
			format: T::FORMAT,
			width: self.width,
			height: self.height,
			props: Default::default(),
		}
	}
	#[inline]
	fn data(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<T: PixelFormat + Copy + dataview::Pod> ImageToTexture for (&Image<T>, &crate::TextureProps) {
	#[inline]
	fn info(&self) -> crate::Texture2DInfo {
		let (image, &props) = self;
		crate::Texture2DInfo {
			format: T::FORMAT,
			width: image.width,
			height: image.height,
			props,
		}
	}
	#[inline]
	fn data(&self) -> &[u8] {
		self.0.as_bytes()
	}
}
