/*!
Utility to load PNG files to texture.
*/

use std::fs;

#[derive(Debug)]
pub enum LoadError {
	Gfx(crate::GfxError),
	PNG(png::DecodingError),
}

impl From<crate::GfxError> for LoadError {
	#[inline]
	fn from(e: crate::GfxError) -> Self {
		LoadError::Gfx(e)
	}
}
impl From<png::DecodingError> for LoadError {
	#[inline]
	fn from(e: png::DecodingError) -> Self {
		LoadError::PNG(e)
	}
}

/// Texture properties.
pub struct TextureProps {
	pub filter_min: crate::TextureFilter,
	pub filter_mag: crate::TextureFilter,
	pub wrap_u: crate::TextureWrap,
	pub wrap_v: crate::TextureWrap,
}

/// With a texture sprite sheet tightly packed, add a 1px gutter around each sprite.
/// The gutter is a copy of the edge pixels of the sprite.
pub fn gutter(sprite_width: usize, sprite_height: usize, bytes_per_pixel: usize) -> impl FnMut(&mut Vec<u8>, &mut png::OutputInfo) {
	move |image, info| {
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

pub fn load(
	g: &mut crate::Graphics,
	name: Option<&str>,
	path: &str,
	props: &TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut png::OutputInfo)>,
) -> Result<crate::Texture2D, LoadError> {

	// Read the PNG file
	let file = fs::File::open(path).map_err(png::DecodingError::IoError)?;
	let mut decoder = png::Decoder::new(file);
	decoder.set_transformations(png::Transformations::normalize_to_color8());
	let mut reader = decoder.read_info()?;
	let mut pixels = vec![0; reader.output_buffer_size()];
	let mut info = reader.next_frame(&mut pixels)?;

	// Only support 8-bit Rgba images
	assert_eq!(info.bit_depth, png::BitDepth::Eight);
	assert_eq!(info.color_type, png::ColorType::Rgba);

	if let Some(transform) = transform {
		transform(&mut pixels, &mut info);
	}

	let tx = g.texture2d_create(name, &crate::Texture2DInfo {
		width: info.width as i32,
		height: info.height as i32,
		format: crate::TextureFormat::R8G8B8A8,
		filter_min: props.filter_min,
		filter_mag: props.filter_mag,
		wrap_u: props.wrap_u,
		wrap_v: props.wrap_v,
		border_color: [0, 0, 0, 0],
	})?;
	g.texture2d_set_data(tx, &pixels)?;
	Ok(tx)
}
