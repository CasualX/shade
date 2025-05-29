/*!
Utility to load GIF files to texture.
*/

use std::fs;
use std::path;

use super::{AnimatedImage, TextureProps};

#[derive(Debug)]
pub enum LoadError {
	Gfx(crate::GfxError),
	GIF(gif::DecodingError),
}

impl From<crate::GfxError> for LoadError {
	#[inline]
	fn from(e: crate::GfxError) -> Self {
		LoadError::Gfx(e)
	}
}
impl From<gif::DecodingError> for LoadError {
	#[inline]
	fn from(e: gif::DecodingError) -> Self {
		LoadError::GIF(e)
	}
}

pub fn load_textures(
	g: &mut crate::Graphics,
	name: Option<&str>,
	path: impl AsRef<path::Path>,
	props: &TextureProps,
) -> Result<AnimatedImage, LoadError> {
	_load_textures(g, name, path.as_ref(), props)
}

fn _load_textures(
	g: &mut crate::Graphics,
	name: Option<&str>,
	path: &path::Path,
	props: &TextureProps,
) -> Result<AnimatedImage, LoadError> {
	// Read the GIF file
	let file = fs::File::open(path).map_err(gif::DecodingError::Io)?;

	let mut opts = gif::DecodeOptions::new();
	opts.set_color_output(gif::ColorOutput::RGBA);
	let mut decoder = opts.read_info(file)?;

	let width = decoder.width();
	let height = decoder.height();
	let repeat = match decoder.repeat() {
		gif::Repeat::Infinite => true,
		gif::Repeat::Finite(count) => count > 0,
	};
	let frame_size = width as usize * height as usize * 4;
	let info = crate::Texture2DInfo {
		width: width as i32,
		height: height as i32,
		format: crate::TextureFormat::RGBA8,
		filter_min: props.filter_min,
		filter_mag: props.filter_mag,
		wrap_u: props.wrap_u,
		wrap_v: props.wrap_v,
		border_color: [0, 0, 0, 0],
	};

	let mut frames = Vec::new();
	let mut length_10ms = 0;
	let mut delay_10ms = !0u32;
	while let Some(frame) = decoder.read_next_frame()? {
		// Process every frame
		let pixels = frame.buffer.as_ref();
		assert_eq!(pixels.len(), frame_size);

		// Use a fixed delay for each frame...
		length_10ms += frame.delay as i32;
		delay_10ms = u32::min(delay_10ms, frame.delay as u32);

		let tex = g.texture2d_create(name, &info)?;
		g.texture2d_set_data(tex, pixels)?;
		frames.push(tex);
	}

	let width = width as i32;
	let height = height as i32;
	let delay = delay_10ms as i32 as f32 / 100.0; // Convert to seconds
	let length = length_10ms as i32 as f32 / 100.0; // Convert to seconds
	Ok(AnimatedImage { width, height, frames, delay, length, repeat })
}
