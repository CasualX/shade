use std::{fs, io, path};
use cvmath::Bounds2;

use super::{AnimatedImage, DecodedImage, Image, ImageRGBA};

pub const GIF_SIGNATURE_87A: &[u8] = b"GIF87a";
pub const GIF_SIGNATURE_89A: &[u8] = b"GIF89a";

struct GifDecoderState {
	canvas: ImageRGBA,
	background: [u8; 4],
	disposal: gif::DisposalMethod,
	disposal_rect: Bounds2<i32>,
	restore: Option<Vec<[u8; 4]>>,
}

fn checked_frame_rect(canvas: &ImageRGBA, frame: &gif::Frame<'_>) -> Result<Bounds2<i32>, gif::DecodingError> {
	let left = frame.left as i32;
	let top = frame.top as i32;
	let width = frame.width as i32;
	let height = frame.height as i32;
	let right = left + width;
	let bottom = top + height;
	if right > canvas.width || bottom > canvas.height {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "frame rectangle exceeds GIF canvas").into());
	}
	Ok(Bounds2!(left, top, right, bottom))
}

fn composite_frame(canvas: &mut ImageRGBA, frame: &gif::Frame<'_>) -> Result<Bounds2<i32>, gif::DecodingError> {
	let rect = checked_frame_rect(canvas, frame)?;
	let width = (rect.maxs.x - rect.mins.x) as usize;
	let height = (rect.maxs.y - rect.mins.y) as usize;
	let pixels = frame.buffer.as_ref();
	let pixel_count = width * height;
	if pixels.len() % 4 != 0 || pixels.len() / 4 != pixel_count {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "frame buffer length does not match frame rectangle").into());
	}

	for y in 0..height {
		let src_row = y * width * 4;
		let dst_row = (rect.mins.y as usize + y) * canvas.width as usize + rect.mins.x as usize;
		for x in 0..width {
			let src = src_row + x * 4;
			if pixels[src + 3] != 0 {
				canvas.data[dst_row + x] = [pixels[src], pixels[src + 1], pixels[src + 2], pixels[src + 3]];
			}
		}
	}

	Ok(rect)
}

impl GifDecoderState {
	fn apply_disposal(&mut self) {
		match self.disposal {
			gif::DisposalMethod::Background => {
				for y in self.disposal_rect.mins.y..self.disposal_rect.maxs.y {
					let row = y as usize * self.canvas.width as usize;
					for x in self.disposal_rect.mins.x..self.disposal_rect.maxs.x {
						self.canvas.data[row + x as usize] = self.background;
					}
				}
			}
			gif::DisposalMethod::Previous => {
				if let Some(restore) = self.restore.as_deref() {
					self.canvas.data.copy_from_slice(restore);
				}
			}
			gif::DisposalMethod::Any | gif::DisposalMethod::Keep => {}
		}
	}

	fn snapshot_restore(&mut self) {
		match &mut self.restore {
			Some(restore) => restore.clone_from(&self.canvas.data),
			None => self.restore = Some(self.canvas.data.clone()),
		}
	}

	fn into_canvas(self) -> ImageRGBA {
		self.canvas
	}
}

//----------------------------------------------------------------

fn decoder_init<R: io::Read>(stream: R) -> Result<gif::Decoder<R>, gif::DecodingError> {
	let mut opts = gif::DecodeOptions::new();
	opts.set_color_output(gif::ColorOutput::RGBA);
	opts.read_info(stream)
}

fn decoder_repeats<R: io::Read>(decoder: &gif::Decoder<R>) -> bool {
	match decoder.repeat() {
		gif::Repeat::Infinite => true,
		gif::Repeat::Finite(count) => count > 0,
	}
}

fn decoder_background_color<R: io::Read>(decoder: &gif::Decoder<R>) -> [u8; 4] {
	let Some(bg_color) = decoder.bg_color() else {
		return [0, 0, 0, 0];
	};
	let Some(palette) = decoder.global_palette() else {
		return [0, 0, 0, 0];
	};
	let Some(rgb) = palette.get(bg_color * 3..bg_color * 3 + 3) else {
		return [0, 0, 0, 0];
	};
	[rgb[0], rgb[1], rgb[2], 255]
}

fn decoder_state<R: io::Read>(decoder: &gif::Decoder<R>) -> GifDecoderState {
	let width = decoder.width() as i32;
	let height = decoder.height() as i32;
	let len = (width as usize) * (height as usize);
	let data = vec![[0, 0, 0, 0]; len];
	let canvas = Image { width, height, data };
	let background = decoder_background_color(&decoder);
	GifDecoderState {
		canvas,
		background,
		disposal: gif::DisposalMethod::Any,
		disposal_rect: Bounds2::ZERO,
		restore: None,
	}
}

fn render_next_frame<R: io::Read>(decoder: &mut gif::Decoder<R>, state: &mut GifDecoderState) -> Result<Option<f32>, gif::DecodingError> {
	let Some(frame) = decoder.read_next_frame()? else {
		return Ok(None);
	};

	state.apply_disposal();

	let delay = frame.delay as i32 as f32 * 0.01;
	if frame.dispose == gif::DisposalMethod::Previous {
		state.snapshot_restore();
	}
	state.disposal_rect = composite_frame(&mut state.canvas, frame)?;
	state.disposal = frame.dispose;

	Ok(Some(delay))
}

//----------------------------------------------------------------

fn load_animated_file(path: &path::Path) -> Result<AnimatedImage, gif::DecodingError> {
	let file = fs::File::open(path).map_err(gif::DecodingError::Io)?;
	let reader = io::BufReader::new(file);
	load_animated_stream(reader)
}

fn load_animated_stream(stream: impl io::Read) -> Result<AnimatedImage, gif::DecodingError> {
	let mut decoder = decoder_init(stream)?;
	let mut state = decoder_state(&decoder);
	let width = state.canvas.width;
	let height = state.canvas.height;
	let repeat = decoder_repeats(&decoder);
	let mut frames = Vec::new();
	let mut delays = Vec::new();
	while let Some(delay) = render_next_frame(&mut decoder, &mut state)? {
		delays.push(delay);
		frames.push(state.canvas.clone());
	}
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

fn load_single_file(path: &path::Path) -> Result<ImageRGBA, gif::DecodingError> {
	let file = fs::File::open(path).map_err(gif::DecodingError::Io)?;
	let reader = io::BufReader::new(file);
	load_single_stream(reader)
}

fn load_single_stream(stream: impl io::Read) -> Result<ImageRGBA, gif::DecodingError> {
	let mut decoder = decoder_init(stream)?;
	let mut state = decoder_state(&decoder);
	let mut saw_frame = false;
	while render_next_frame(&mut decoder, &mut state)?.is_some() {
		saw_frame = true;
	}
	if !saw_frame {
		return Err(gif::DecodingError::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "No frames in GIF")));
	}
	Ok(state.into_canvas())
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

#[cfg(test)]
mod tests;
