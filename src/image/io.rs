//! Reading and writing image files.

use super::*;

#[cfg(feature = "png")]
pub mod png;

#[cfg(feature = "gif")]
pub mod gif;

#[cfg(feature = "jpeg")]
pub mod jpeg;

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadImageError {
	/// An I/O error occurred.
	Io(std::io::Error),
	#[cfg(feature = "png")]
	/// An error occurred while decoding a PNG image.
	PNG(::png::DecodingError),
	#[cfg(feature = "gif")]
	/// An error occurred while decoding a GIF image.
	GIF(::gif::DecodingFormatError),
	/// An error occurred while decoding a JPEG image.
	#[cfg(feature = "jpeg")]
	JPEG(::zune_jpeg::errors::DecodeErrors),
	/// The image format is not supported.
	UnsupportedFormat,
}

impl From<std::io::Error> for LoadImageError {
	#[inline]
	fn from(e: std::io::Error) -> Self {
		LoadImageError::Io(e)
	}
}

#[cfg(feature = "png")]
impl From<::png::DecodingError> for LoadImageError {
	#[inline]
	fn from(e: ::png::DecodingError) -> Self {
		match e {
			::png::DecodingError::IoError(e) => LoadImageError::Io(e),
			e => LoadImageError::PNG(e),
		}
	}
}

#[cfg(feature = "gif")]
impl From<::gif::DecodingError> for LoadImageError {
	#[inline]
	fn from(e: ::gif::DecodingError) -> Self {
		match e {
			::gif::DecodingError::Io(e) => LoadImageError::Io(e),
			::gif::DecodingError::Format(e) => LoadImageError::GIF(e),
		}
	}
}

#[cfg(feature = "jpeg")]
impl From<::zune_jpeg::errors::DecodeErrors> for LoadImageError {
	#[inline]
	fn from(e: ::zune_jpeg::errors::DecodeErrors) -> Self {
		match e {
			::zune_jpeg::errors::DecodeErrors::IoErrors(::zune_jpeg::zune_core::bytestream::ZByteIoError::StdIoError(e)) => LoadImageError::Io(e),
			e => LoadImageError::JPEG(e),
		}
	}
}
