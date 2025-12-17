use std::{fs, io, path};

use super::{AnimatedImage, DecodedImage, Image};

//----------------------------------------------------------------
// Animated GIF loading

fn load_animated_file(path: &path::Path) -> Result<AnimatedImage, gif::DecodingError> {
	let file = fs::File::open(path).map_err(gif::DecodingError::Io)?;
	let reader = io::BufReader::new(file);
	load_animated_stream(reader)
}

fn load_animated_stream(stream: impl io::Read) -> Result<AnimatedImage, gif::DecodingError> {
	let mut opts = gif::DecodeOptions::new();
	opts.set_color_output(gif::ColorOutput::RGBA);
	let mut decoder = opts.read_info(stream)?;

	let width = decoder.width() as usize;
	let height = decoder.height() as usize;
	let len = width * height;

	let mut frames = Vec::new();
	let mut delays = Vec::new();
	while let Some(frame) = decoder.read_next_frame()? {
		// Add delay in seconds
		delays.push(frame.delay as i32 as f32 * 0.01);

		// Convert pixel data
		let pixels = frame.buffer.as_ref();
		assert_eq!(pixels.len(), len * 4);
		let mut data = Vec::with_capacity(len);
		for chunk in pixels.chunks_exact(4) {
			data.push([chunk[0], chunk[1], chunk[2], chunk[3]]);
		}

		// Create and add image frame
		let frame = crate::image::Image {
			width: width as i32,
			height: height as i32,
			data,
		};
		frames.push(frame);
	}

	let width = width as i32;
	let height = height as i32;
	let repeat = match decoder.repeat() {
		gif::Repeat::Infinite => true,
		gif::Repeat::Finite(count) => count > 0,
	};
	Ok(AnimatedImage { width, height, frames, delays, repeat })
}

impl AnimatedImage {
	/// Loads a GIF image from a file.
	#[inline]
	pub fn load_file_gif(path: impl AsRef<path::Path>) -> Result<Self, gif::DecodingError> {
		load_animated_file(path.as_ref())
	}
	/// Loads a GIF image from an I/O stream.
	#[inline]
	pub fn load_stream_gif(stream: impl io::Read) -> Result<Self, gif::DecodingError> {
		load_animated_stream(stream)
	}
	/// Loads a GIF image from memory.
	#[inline]
	pub fn load_memory_gif(mut data: &[u8]) -> Result<Self, gif::DecodingError> {
		load_animated_stream(&mut data)
	}
}

//----------------------------------------------------------------
// Single-frame GIF loading

fn load_single_file(path: &path::Path) -> Result<Image<[u8; 4]>, gif::DecodingError> {
	let file = fs::File::open(path).map_err(gif::DecodingError::Io)?;
	let reader = io::BufReader::new(file);
	load_single_stream(reader)
}

fn load_single_stream(stream: impl io::Read) -> Result<Image<[u8; 4]>, gif::DecodingError> {
	let mut opts = gif::DecodeOptions::new();
	opts.set_color_output(gif::ColorOutput::RGBA);
	let mut decoder = opts.read_info(stream)?;

	let width = decoder.width() as usize;
	let height = decoder.height() as usize;
	let len = width * height;

	let Some(frame) = decoder.read_next_frame()? else {
		return Err(gif::DecodingError::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "No frames in GIF")));
	};

	// Convert pixel data
	let pixels = frame.buffer.as_ref();
	assert_eq!(pixels.len(), len * 4);
	let mut data = Vec::with_capacity(len);
	for chunk in pixels.chunks_exact(4) {
		data.push([chunk[0], chunk[1], chunk[2], chunk[3]]);
	}

	Ok(Image {
		width: width as i32,
		height: height as i32,
		data,
	})
}

impl DecodedImage {
	/// Loads a GIF image from a file.
	#[inline]
	pub fn load_file_gif(path: impl AsRef<path::Path>) -> Result<Self, gif::DecodingError> {
		let image = load_single_file(path.as_ref())?;
		Ok(DecodedImage::RGBA(image))
	}
	/// Loads a GIF image from an I/O stream.
	#[inline]
	pub fn load_stream_gif(stream: impl io::Read) -> Result<Self, gif::DecodingError> {
		let image = load_single_stream(stream)?;
		Ok(DecodedImage::RGBA(image))
	}
	/// Loads a GIF image from memory.
	#[inline]
	pub fn load_memory_gif(mut data: &[u8]) -> Result<Self, gif::DecodingError> {
		let image = load_single_stream(&mut data)?;
		Ok(DecodedImage::RGBA(image))
	}
}

impl<T> Image<T> where Image<T>: From<DecodedImage> {
	/// Loads a GIF image from a file.
	#[inline]
	pub fn load_file_gif(path: impl AsRef<path::Path>) -> Result<Self, gif::DecodingError> {
		let decoded = DecodedImage::load_file_gif(path)?;
		Ok(decoded.into())
	}
	/// Loads a GIF image from an I/O stream.
	#[inline]
	pub fn load_stream_gif(stream: impl io::Read) -> Result<Self, gif::DecodingError> {
		let decoded = DecodedImage::load_stream_gif(stream)?;
		Ok(decoded.into())
	}
	/// Loads a GIF image from memory.
	#[inline]
	pub fn load_memory_gif(mut data: &[u8]) -> Result<Self, gif::DecodingError> {
		let decoded = DecodedImage::load_memory_gif(&mut data)?;
		Ok(decoded.into())
	}
}
