/*!
Utility to load PNG files to texture.
*/

use std::{fs, io, path};

use super::{ImageSize, TextureProps};

pub type LoadError = png::DecodingError;

#[inline]
pub fn load_file(
	g: &mut crate::Graphics,
	name: Option<&str>,
	path: impl AsRef<path::Path>,
	props: &TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut ImageSize)>,
) -> Result<crate::Texture2D, LoadError> {
	let mut file = fs::File::open(path).map_err(png::DecodingError::IoError)?;
	load_stream(g, name, &mut file, props, transform)
}

pub fn load_stream(
	g: &mut crate::Graphics,
	name: Option<&str>,
	stream: &mut dyn io::Read,
	props: &TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut ImageSize)>,
) -> Result<crate::Texture2D, LoadError> {

	// Read the PNG file
	let mut decoder = png::Decoder::new(stream);
	decoder.set_transformations(png::Transformations::normalize_to_color8());
	let mut reader = decoder.read_info()?;
	let mut pixels = vec![0; reader.output_buffer_size()];
	let mut info = reader.next_frame(&mut pixels)?;

	// Only support 8-bit Rgba images
	assert_eq!(info.bit_depth, png::BitDepth::Eight);
	let format = match info.color_type {
		png::ColorType::Rgb => crate::TextureFormat::RGB8,
		png::ColorType::Rgba => crate::TextureFormat::RGBA8,
		png::ColorType::Grayscale => crate::TextureFormat::Grey8,
		_ => unimplemented!("Unsupported PNG color type: {:?}", info.color_type),
	};

	if let Some(transform) = transform {
		let mut size = ImageSize {
			width: info.width,
			height: info.height,
			bytes_per_pixel: match info.color_type {
				png::ColorType::Grayscale => 1,
				png::ColorType::Rgb => 3,
				png::ColorType::Indexed => 1,
				png::ColorType::GrayscaleAlpha => 2,
				png::ColorType::Rgba => 4,
			},
		};
		transform(&mut pixels, &mut size);
		info.width = size.width;
		info.height = size.height;
	}

	let tx = g.texture2d_create(name, &crate::Texture2DInfo {
		width: info.width as i32,
		height: info.height as i32,
		format,
		filter_min: props.filter_min,
		filter_mag: props.filter_mag,
		wrap_u: props.wrap_u,
		wrap_v: props.wrap_v,
		border_color: [0, 0, 0, 0],
	});
	g.texture2d_set_data(tx, &pixels);
	Ok(tx)
}
