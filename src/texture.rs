
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
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
pub struct TextureProps {
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
}

impl TextureProps {
	#[inline]
	pub const fn from(filter: TextureFilter, wrap: TextureWrap) -> TextureProps {
		TextureProps {
			filter_min: filter,
			filter_mag: filter,
			wrap_u: wrap,
			wrap_v: wrap,
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
	pub levels: u8,
	pub width: i32,
	pub height: i32,
	pub props: TextureProps,
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
