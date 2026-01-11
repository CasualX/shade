use crate::color::PixelFormat;
use super::*;

impl crate::ImageToTexture for DecodedImage {
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

impl crate::ImageToTexture for (&DecodedImage, &crate::TextureProps) {
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

impl<T: PixelFormat + Copy + dataview::Pod> crate::ImageToTexture for Image<T> {
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

impl<T: PixelFormat + Copy + dataview::Pod> crate::ImageToTexture for (&Image<T>, &crate::TextureProps) {
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
