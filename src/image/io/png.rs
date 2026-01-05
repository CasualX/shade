use std::{fs, io, mem, path};

use super::{AnimatedImage, DecodedImage, Image};

pub const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

//----------------------------------------------------------------
// Loading PNG files

fn load_file(path: &path::Path) -> Result<DecodedImage, png::DecodingError> {
	let file = fs::File::open(path).map_err(png::DecodingError::IoError)?;
	let reader = io::BufReader::new(file);
	load_stream(reader)
}

fn load_stream(stream: impl io::Read) -> Result<DecodedImage, png::DecodingError> {
	// Read the PNG file
	let mut decoder = png::Decoder::new(stream);
	decoder.set_transformations(png::Transformations::normalize_to_color8());
	let mut reader = decoder.read_info()?;
	let mut pixels = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut pixels)?;

	// Only support 8-bit images
	if !matches!(info.bit_depth, png::BitDepth::Eight) {
		return Err(png::DecodingError::IoError(io::Error::new(
			io::ErrorKind::InvalidData,
			format!("Unsupported PNG bit depth: {:?}", info.bit_depth),
		)));
	}

	let image = match info.color_type {
		png::ColorType::Rgb => {
			let image = Image {
				width: info.width as i32,
				height: info.height as i32,
				data: convert_from_rgb(&info, pixels),
			};
			DecodedImage::RGB(image)
		}
		png::ColorType::Rgba => {
			let image = Image {
				width: info.width as i32,
				height: info.height as i32,
				data: convert_from_rgba(&info, pixels),
			};
			DecodedImage::RGBA(image)
		}
		png::ColorType::Grayscale => {
			let image = Image {
				width: info.width as i32,
				height: info.height as i32,
				data: convert_from_grayscale(&info, pixels),
			};
			DecodedImage::Grey(image)
		}
		_ => unimplemented!("Unsupported PNG color type: {:?}", info.color_type),
	};

	Ok(image)
}

fn convert_from_rgb(info: &png::OutputInfo, mut pixels: Vec<u8>) -> Vec<[u8; 3]> {
	assert_eq!(pixels.len(), info.width as usize * info.height as usize * 3);
	assert!(pixels.capacity() % 3 == 0);
	let ptr = pixels.as_mut_ptr() as *mut [u8; 3];
	let len = pixels.len() / 3;
	let capacity = pixels.capacity() / 3;
	let new_vec = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
	mem::forget(pixels);
	new_vec
}
fn convert_from_rgba(info: &png::OutputInfo, pixels: Vec<u8>) -> Vec<[u8; 4]> {
	assert_eq!(pixels.len(), info.width as usize * info.height as usize * 4);
	assert!(pixels.capacity() % 4 == 0);
	let ptr = pixels.as_ptr() as *mut [u8; 4];
	let len = pixels.len() / 4;
	let capacity = pixels.capacity() / 4;
	let new_vec = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
	mem::forget(pixels);
	new_vec
}
fn convert_from_grayscale(info: &png::OutputInfo, pixels: Vec<u8>) -> Vec<u8> {
	assert_eq!(pixels.len(), info.width as usize * info.height as usize);
	pixels
}

impl DecodedImage {
	/// Loads a PNG image from a file.
	#[inline]
	pub fn load_file_png(path: impl AsRef<path::Path>) -> Result<Self, png::DecodingError> {
		load_file(path.as_ref())
	}
	/// Loads a PNG image from an I/O stream.
	#[inline]
	pub fn load_stream_png(stream: impl io::Read) -> Result<Self, png::DecodingError> {
		load_stream(stream)
	}
	/// Loads a PNG image from memory.
	#[inline]
	pub fn load_memory_png(mut data: &[u8]) -> Result<Self, png::DecodingError> {
		load_stream(&mut data)
	}
}

impl<T> Image<T> where Image<T>: From<DecodedImage>{
	/// Loads a PNG image from a file.
	#[inline]
	pub fn load_file_png(path: impl AsRef<path::Path>) -> Result<Self, png::DecodingError> {
		let decoded = DecodedImage::load_file_png(path)?;
		Ok(decoded.into())
	}
	/// Loads a PNG image from an I/O stream.
	#[inline]
	pub fn load_stream_png(stream: impl io::Read) -> Result<Self, png::DecodingError> {
		let decoded = DecodedImage::load_stream_png(stream)?;
		Ok(decoded.into())
	}
	/// Loads a PNG image from memory.
	#[inline]
	pub fn load_memory_png(mut data: &[u8]) -> Result<Self, png::DecodingError> {
		let decoded = DecodedImage::load_memory_png(&mut data)?;
		Ok(decoded.into())
	}
}

impl AnimatedImage {
	/// Loads a PNG image from a file.
	#[inline]
	pub fn load_file_png(path: impl AsRef<path::Path>) -> Result<Self, png::DecodingError> {
		let image = DecodedImage::load_file_png(path)?;
		let image = image.to_rgba();
		Ok(AnimatedImage {
			width: image.width,
			height: image.height,
			frames: vec![image],
			delays: Vec::new(),
			repeat: false,
		})
	}
	/// Loads a PNG image from an I/O stream.
	#[inline]
	pub fn load_stream_png(stream: impl io::Read) -> Result<Self, png::DecodingError> {
		let image = DecodedImage::load_stream_png(stream)?;
		let image = image.to_rgba();
		Ok(AnimatedImage {
			width: image.width,
			height: image.height,
			frames: vec![image],
			delays: Vec::new(),
			repeat: false,
		})
	}
	/// Loads a PNG image from memory.
	#[inline]
	pub fn load_memory_png(mut data: &[u8]) -> Result<Self, png::DecodingError> {
		let image = DecodedImage::load_memory_png(&mut data)?;
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

//----------------------------------------------------------------
// Saving PNG files

fn save_file(path: &path::Path, color: png::ColorType, width: u32, height: u32, data: &[u8]) -> Result<(), png::EncodingError> {
	let file = fs::File::create(path).map_err(png::EncodingError::IoError)?;
	let writer = io::BufWriter::new(file);
	save_stream(writer, color, width, height, data)
}

#[inline]
fn save_stream(stream: impl io::Write, color: png::ColorType, width: u32, height: u32, data: &[u8]) -> Result<(), png::EncodingError> {
	let mut encoder = png::Encoder::new(stream, width, height);
	encoder.set_color(color);
	encoder.set_depth(png::BitDepth::Eight);
	let mut writer = encoder.write_header()?;
	writer.write_image_data(data)?;
	Ok(())
}

impl Image<[u8; 4]> {
	/// Saves the image as a PNG file.
	#[inline]
	pub fn save_file_png(&self, path: impl AsRef<path::Path>) -> Result<(), png::EncodingError> {
		let data = dataview::bytes(self.data.as_slice());
		save_file(path.as_ref(), png::ColorType::Rgba, self.width as u32, self.height as u32, data)
	}
}

impl Image<[u8; 3]> {
	/// Saves the image as a PNG file.
	#[inline]
	pub fn save_file_png(&self, path: impl AsRef<path::Path>) -> Result<(), png::EncodingError> {
		let data = dataview::bytes(self.data.as_slice());
		save_file(path.as_ref(), png::ColorType::Rgb, self.width as u32, self.height as u32, data)
	}
}

impl Image<[u8; 2]> {
	/// Saves the image as a PNG file.
	#[inline]
	pub fn save_file_png(&self, path: impl AsRef<path::Path>) -> Result<(), png::EncodingError> {
		let data = dataview::bytes(self.data.as_slice());
		save_file(path.as_ref(), png::ColorType::GrayscaleAlpha, self.width as u32, self.height as u32, data)
	}
}

impl Image<u8> {
	/// Saves the image as a PNG file.
	#[inline]
	pub fn save_file_png(&self, path: impl AsRef<path::Path>) -> Result<(), png::EncodingError> {
		let data = dataview::bytes(self.data.as_slice());
		save_file(path.as_ref(), png::ColorType::Grayscale, self.width as u32, self.height as u32, data)
	}
}
