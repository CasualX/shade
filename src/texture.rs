
/// Texture format.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[non_exhaustive]
pub enum TextureFormat {
	// Color formats
	#[default]
	RGBA8,
	RGB8,
	R8,

	// Depth formats
	Depth16,
	Depth24,
	Depth32F,
	Depth24Stencil8,
}

impl TextureFormat {
	/// Returns the number of bytes per pixel for the format.
	#[inline]
	pub const fn bytes_per_pixel(self) -> usize {
		match self {
			TextureFormat::RGBA8 => 4,
			TextureFormat::RGB8 => 3,
			TextureFormat::R8 => 1,
			TextureFormat::Depth16 => 2,
			TextureFormat::Depth24 => 4, // Although the internal format is 24-bit, write/readback uses 32-bit values
			TextureFormat::Depth32F => 4,
			TextureFormat::Depth24Stencil8 => 4,
		}
	}
}

/// Texture wrap mode.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
pub enum TextureWrap {
	#[default]
	Edge,
	Border,
	Repeat,
	Mirror,
}

/// Texture filter mode.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
pub enum TextureFilter {
	#[default]
	Linear,
	Nearest,
}

/// Texture properties.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TextureProps {
	pub mip_levels: u8,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
}

impl Default for TextureProps {
	#[inline]
	fn default() -> TextureProps {
		TextureProps {
			mip_levels: 1,
			filter_min: TextureFilter::Linear,
			filter_mag: TextureFilter::Linear,
			wrap_u: TextureWrap::Edge,
			wrap_v: TextureWrap::Edge,
		}
	}
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
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
pub struct Texture2DInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub height: i32,
	pub props: TextureProps,
}

#[inline]
const fn mip_size(dim: i32, level: u8) -> i32 {
	let v = dim >> level;
	if v == 0 { 1 } else { v }
}

impl Texture2DInfo {
	/// Returns the (width, height, byte_size) of the specified mip level.
	#[inline]
	pub const fn mip_size(&self, level: u8) -> (i32, i32, usize) {
		let w = mip_size(self.width, level);
		let h = mip_size(self.height, level);
		(w as i32, h as i32, w as usize * h as usize * self.format.bytes_per_pixel())
	}
}

/// Animated Texture2D structure.
#[derive(Clone, Debug)]
pub struct AnimatedTexture2D {
	/// The width of the image.
	pub width: i32,
	/// The height of the image.
	pub height: i32,
	/// The frames of the image.
	pub frames: Vec<Texture2D>,
	/// The length of the animation in seconds.
	pub length: f32,
	/// If the animation should repeat.
	pub repeat: bool,
}

impl AnimatedTexture2D {
	pub fn get_frame(&self, time: f64) -> Texture2D {
		if self.frames.is_empty() {
			return Texture2D::INVALID;
		}
		if self.length <= 0.0 {
			return self.frames[0];
		}
		let fract = if self.repeat {
			f64::fract(time / self.length as f64)
		}
		else {
			f64::min(time, self.length as f64) / self.length as f64
		};
		let index = usize::min(f64::floor(fract * self.frames.len() as i32 as f64) as i32 as usize, self.frames.len() - 1);
		self.frames[index]
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
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
pub struct TextureCubeInfo {
	pub format: TextureFormat,
	pub width: i32,
	pub height: i32,
	pub props: TextureProps,
}
