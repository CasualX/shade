
/// Texture format.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum TextureFormat {
	RGB8,
	RGBA8,
	Grey8,
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

//----------------------------------------------------------------
// Texture1D handle.
/*
define_handle!(Texture1D);

/// Texture1D information.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct Texture1DInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub border_color: [u8; 4],
}

impl Default for Texture1DInfo {
	fn default() -> Self {
		Self {
			format: TextureFormat::RGBA8,
			width: 0,
			filter_min: TextureFilter::Linear,
			filter_mag: TextureFilter::Linear,
			wrap_u: TextureWrap::ClampEdge,
			border_color: [0, 0, 0, 0],
		}
	}
}*/

//----------------------------------------------------------------
// Texture2D handle.

define_handle!(Texture2D);

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
			format: TextureFormat::RGBA8,
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

//-----------------------------------------------------------------
// Texture2DArray handle.
/*
define_handle!(Texture2DArray);

/// Texture2DArray information.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct Texture2DArrayInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub height: i32,
	pub count: u16,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
	pub border_color: [u8; 4],
}

impl Default for Texture2DArrayInfo {
	fn default() -> Self {
		Self {
			format: TextureFormat::RGBA8,
			width: 0,
			height: 0,
			count: 0,
			filter_min: TextureFilter::Linear,
			filter_mag: TextureFilter::Linear,
			wrap_u: TextureWrap::ClampEdge,
			wrap_v: TextureWrap::ClampEdge,
			border_color: [0, 0, 0, 0],
		}
	}
}*/

//-------------------------------------------------------------------
// TextureCube handle.

define_handle!(TextureCube);

/// TextureCube information.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TextureCubeInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub height: i32,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
	pub border_color: [u8; 4],
}

impl Default for TextureCubeInfo {
	fn default() -> Self {
		Self {
			format: TextureFormat::RGBA8,
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
