/*!
Utilities for loading and manipulating images.
*/

/// Texture properties.
pub struct TextureProps {
	pub filter_min: crate::TextureFilter,
	pub filter_mag: crate::TextureFilter,
	pub wrap_u: crate::TextureWrap,
	pub wrap_v: crate::TextureWrap,
}

pub struct ImageSize {
	pub width: u32,
	pub height: u32,
	pub bytes_per_pixel: usize,
}

/// With a texture sprite sheet tightly packed, add a 1px gutter around each sprite.
/// The gutter is a copy of the edge pixels of the sprite.
pub fn gutter(sprite_width: usize, sprite_height: usize) -> impl FnMut(&mut Vec<u8>, &mut ImageSize) {
	move |image, info| {
		let bytes_per_pixel = info.bytes_per_pixel;
		let nsprites_x = info.width as usize / sprite_width;
		let nsprites_y = info.height as usize / sprite_height;
		let mut new_image = Vec::new();
		let new_width = nsprites_x * (sprite_width + 2);
		let new_height = nsprites_y * (sprite_height + 2);
		let line_size = info.width as usize * bytes_per_pixel;
		let new_line_size = new_width * bytes_per_pixel;
		new_image.resize(new_width * new_height * bytes_per_pixel, 0);
		for sprite_y in 0..nsprites_y {
			for sprite_x in 0..nsprites_x {
				let mut copy_line = |src_y, dst_y| {
					let mut src_idx = src_y * line_size + sprite_x * sprite_width * bytes_per_pixel;
					let mut dst_idx = dst_y * new_line_size + sprite_x * (sprite_width + 2) * bytes_per_pixel;

					// Copy the left gutter
					new_image[dst_idx..dst_idx + bytes_per_pixel].copy_from_slice(&image[src_idx..src_idx + bytes_per_pixel]);
					// Copy the sprite line
					dst_idx += bytes_per_pixel;
					new_image[dst_idx..dst_idx + sprite_width * bytes_per_pixel].copy_from_slice(&image[src_idx..src_idx + sprite_width * bytes_per_pixel]);
					// Copy the right gutter
					src_idx += (sprite_width - 1) * bytes_per_pixel;
					dst_idx += sprite_width * bytes_per_pixel;
					new_image[dst_idx..dst_idx + bytes_per_pixel].copy_from_slice(&image[src_idx..src_idx + bytes_per_pixel]);
				};

				copy_line(sprite_y * sprite_height, sprite_y * (sprite_height + 2));
				for line_y in 0..sprite_height {
					copy_line(sprite_y * sprite_height + line_y, sprite_y * (sprite_height + 2) + line_y + 1);
				}
				copy_line(sprite_y * sprite_height + sprite_height - 1, sprite_y * (sprite_height + 2) + sprite_height + 1);
			}
		}
		*image = new_image;
		info.width = new_width as u32;
		info.height = new_height as u32;
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadImageError {
	/// An error occurred while loading the image.
	Gfx(crate::GfxError),
	#[cfg(feature = "png")]
	/// An error occurred while decoding a PNG image.
	PNG(::png::DecodingError),
	#[cfg(feature = "gif")]
	/// An error occurred while decoding a GIF image.
	GIF(::gif::DecodingError),
	/// The image format is not supported.
	UnsupportedFormat,
}

impl From<crate::GfxError> for LoadImageError {
	#[inline]
	fn from(e: crate::GfxError) -> Self {
		LoadImageError::Gfx(e)
	}
}

#[cfg(feature = "png")]
impl From<png::LoadError> for LoadImageError {
	#[inline]
	fn from(e: png::LoadError) -> Self {
		match e {
			png::LoadError::Gfx(e) => LoadImageError::Gfx(e),
			png::LoadError::PNG(e) => LoadImageError::PNG(e),
		}
	}
}

#[cfg(feature = "gif")]
impl From<gif::LoadError> for LoadImageError {
	#[inline]
	fn from(e: gif::LoadError) -> Self {
		match e {
			gif::LoadError::Gfx(e) => LoadImageError::Gfx(e),
			gif::LoadError::GIF(e) => LoadImageError::GIF(e),
		}
	}
}

#[derive(Clone, Debug)]
pub struct AnimatedImage {
	/// The width of the image.
	pub width: i32,
	/// The height of the image.
	pub height: i32,
	/// The frames of the image.
	pub frames: Vec<crate::Texture2D>,
	/// The time to display each frame in seconds.
	pub delay: f32,
	/// The length of the animation in seconds.
	pub length: f32,
	/// If the animation should repeat.
	pub repeat: bool,
}

impl AnimatedImage {
	pub fn get_frame(&self, time: f32) -> crate::Texture2D {
		if self.frames.is_empty() {
			return crate::Texture2D::INVALID;
		}
		if self.delay <= 0.0 || self.length <= 0.0 {
			return self.frames[0];
		}
		let fract = if self.repeat {
			f32::fract(time / self.length)
		}
		else {
			f32::min(time, self.length) / self.length
		};
		let index = usize::min(f32::floor(fract * self.frames.len() as i32 as f32) as usize, self.frames.len() - 1);
		self.frames[index]
	}
}

use std::path;
impl AnimatedImage {
	pub fn load(g: &mut crate::Graphics, name: Option<&str>, path: impl AsRef<path::Path>, props: &TextureProps) -> Result<Self, LoadImageError> {
		Self::_load(g, name, path.as_ref(), props)
	}
	fn _load(g: &mut crate::Graphics, name: Option<&str>, path: &path::Path, props: &TextureProps) -> Result<Self, LoadImageError> {
		#[cfg(feature = "png")]
		if path.extension().and_then(|s| s.to_str()) == Some("png") {
			let tex = png::load_file(g, name, path, props, None)?;
			let info = g.texture2d_get_info(tex)?;
			return Ok(AnimatedImage {
				width: info.width,
				height: info.height,
				frames: vec![tex],
				delay: 0.0,
				length: 0.0,
				repeat: false,
			});
		}
		#[cfg(feature = "gif")]
		if path.extension().and_then(|s| s.to_str()) == Some("gif") {
			let anim = gif::load_textures(g, name, path, props)?;
			return Ok(anim)
		};

		Err(LoadImageError::UnsupportedFormat)
	}
}

#[cfg(feature = "png")]
pub mod png;

#[cfg(feature = "gif")]
pub mod gif;
