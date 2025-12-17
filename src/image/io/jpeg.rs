use std::{fs, io, mem, path};

use super::{AnimatedImage, DecodedImage, Image};

use zune_jpeg::{errors::DecodeErrors, JpegDecoder, ImageInfo};
use zune_jpeg::zune_core::bytestream::{ZByteIoError, ZCursor};

//----------------------------------------------------------------
// Loading JPEG files

fn load_file(path: &path::Path) -> Result<DecodedImage, DecodeErrors> {
	let data = fs::read(path).map_err(|e| DecodeErrors::IoErrors(ZByteIoError::StdIoError(e)))?;
	load_memory(&data)
}

fn load_stream(mut stream: impl io::Read) -> Result<DecodedImage, DecodeErrors> {
	let mut data = Vec::new();
	stream.read_to_end(&mut data).map_err(|e| DecodeErrors::IoErrors(ZByteIoError::StdIoError(e)))?;
	load_memory(&data)
}

fn load_memory(data: &[u8]) -> Result<DecodedImage, DecodeErrors> {
	let cursor = ZCursor::new(data);
	let mut decoder = JpegDecoder::new(cursor);
	let pixels = decoder.decode()?;
	let info = decoder
		.info()
		.ok_or(DecodeErrors::FormatStatic("Missing JPEG metadata"))?;

	let image = convert_to_decoded(&info, pixels)?;
	Ok(image)
}

fn convert_to_decoded(info: &ImageInfo, pixels: Vec<u8>) -> Result<DecodedImage, DecodeErrors> {
	let width = info.width as usize;
	let height = info.height as usize;
	let len = width * height;

	if pixels.len() == len {
		let data = convert_from_luma(len, pixels);
		Ok(DecodedImage::Grey(Image {
			width: width as i32,
			height: height as i32,
			data,
		}))
	}
	else if pixels.len() == len * 3 {
		let data = convert_from_rgb(len, pixels);
		Ok(DecodedImage::RGB(Image {
			width: width as i32,
			height: height as i32,
			data,
		}))
	}
	else {
		let message = format!("Unsupported JPEG pixel buffer size {} for {width}x{height} image", pixels.len());
		Err(DecodeErrors::Format(message))
	}
}

fn convert_from_luma(expected_len: usize, pixels: Vec<u8>) -> Vec<u8> {
	assert_eq!(pixels.len(), expected_len);
	pixels
}

fn convert_from_rgb(expected_pixels: usize, mut pixels: Vec<u8>) -> Vec<[u8; 3]> {
	let expected_len = expected_pixels * 3;
	assert_eq!(pixels.len(), expected_len);
	assert!(pixels.capacity() % 3 == 0);
	let ptr = pixels.as_mut_ptr() as *mut [u8; 3];
	let len = pixels.len() / 3;
	let capacity = pixels.capacity() / 3;
	let new_vec = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
	mem::forget(pixels);
	new_vec
}

impl DecodedImage {
	/// Loads a JPEG image from a file.
	#[inline]
	pub fn load_file_jpeg(path: impl AsRef<path::Path>) -> Result<Self, DecodeErrors> {
		load_file(path.as_ref())
	}
	/// Loads a JPEG image from an I/O stream.
	#[inline]
	pub fn load_stream_jpeg(stream: impl io::Read) -> Result<Self, DecodeErrors> {
		load_stream(stream)
	}
	/// Loads a JPEG image from memory.
	#[inline]
	pub fn load_memory_jpeg(mut data: &[u8]) -> Result<Self, DecodeErrors> {
		load_memory(&mut data)
	}
}

impl<T> Image<T> where Image<T>: From<DecodedImage> {
	/// Loads a JPEG image from a file.
	#[inline]
	pub fn load_file_jpeg(path: impl AsRef<path::Path>) -> Result<Self, DecodeErrors> {
		let decoded = DecodedImage::load_file_jpeg(path)?;
		Ok(decoded.into())
	}
	/// Loads a JPEG image from an I/O stream.
	#[inline]
	pub fn load_stream_jpeg(stream: impl io::Read) -> Result<Self, DecodeErrors> {
		let decoded = DecodedImage::load_stream_jpeg(stream)?;
		Ok(decoded.into())
	}
	/// Loads a JPEG image from memory.
	#[inline]
	pub fn load_memory_jpeg(mut data: &[u8]) -> Result<Self, DecodeErrors> {
		let decoded = DecodedImage::load_memory_jpeg(&mut data)?;
		Ok(decoded.into())
	}
}

impl AnimatedImage {
	/// Loads a JPEG image from a file.
	///
	/// Note: JPEG is a single-frame format; the resulting `AnimatedImage` contains exactly one frame.
	#[inline]
	pub fn load_file_jpeg(path: impl AsRef<path::Path>) -> Result<Self, DecodeErrors> {
		let image = DecodedImage::load_file_jpeg(path)?;
		let image = image.to_rgba();
		Ok(AnimatedImage {
			width: image.width,
			height: image.height,
			frames: vec![image],
			delays: Vec::new(),
			repeat: false,
		})
	}
	/// Loads a JPEG image from an I/O stream.
	///
	/// Note: JPEG is a single-frame format; the resulting `AnimatedImage` contains exactly one frame.
	#[inline]
	pub fn load_stream_jpeg(stream: impl io::Read) -> Result<Self, DecodeErrors> {
		let image = DecodedImage::load_stream_jpeg(stream)?;
		let image = image.to_rgba();
		Ok(AnimatedImage {
			width: image.width,
			height: image.height,
			frames: vec![image],
			delays: Vec::new(),
			repeat: false,
		})
	}
	/// Loads a JPEG image from memory.
	///
	/// Note: JPEG is a single-frame format; the resulting `AnimatedImage` contains exactly one frame.
	#[inline]
	pub fn load_memory_jpeg(mut data: &[u8]) -> Result<Self, DecodeErrors> {
		let image = DecodedImage::load_memory_jpeg(&mut data)?;
		let image = image.to_rgba();
		Ok(AnimatedImage {
			width: image.width,
			height: image.height,
			frames: vec![image],
			delays: Vec::new(),
			repeat: false,
		})
	}
}
