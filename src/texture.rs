define_handle!(Texture2D);

/// Texture format.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum TextureFormat {
	R8G8B8A8,
}

/// Texture wrap mode.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum TextureWrap {
	ClampEdge,
	ClampBorder,
	Repeat,
	Mirror,
}

/// Texture filter mode.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum TextureFilter {
	Nearest,
	Linear,
}

/// Texture2D information.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct Texture2DInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub height: i32,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
	pub border_color: [u8; 4],
}

impl Default for Texture2DInfo {
	fn default() -> Self {
		Self {
			format: TextureFormat::R8G8B8A8,
			width: 0,
			height: 0,
			filter_min: TextureFilter::Linear,
			filter_mag: TextureFilter::Linear,
			wrap_u: TextureWrap::ClampEdge,
			wrap_v: TextureWrap::ClampEdge,
			border_color: [0, 0, 0, 0],
		}
	}
}
