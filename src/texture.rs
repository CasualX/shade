use super::*;

/// Texture format.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[non_exhaustive]
pub enum TextureFormat {
	// Srgb formats
	#[default]
	SRGBA8,
	SRGB8,

	// Linear 8-bit color formats
	RGBA8,
	RGB8,
	RG8,
	R8,

	// High precision formats
	RGBA32F,
	RGB32F,
	RG32F,
	R32F,

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
		(match self {
			TextureFormat::SRGBA8 => 4u8,
			TextureFormat::SRGB8 => 3u8,
			TextureFormat::RGBA8 => 4u8,
			TextureFormat::RGB8 => 3u8,
			TextureFormat::RG8 => 2u8,
			TextureFormat::R8 => 1u8,
			TextureFormat::RGBA32F => 16u8,
			TextureFormat::RGB32F => 12u8,
			TextureFormat::RG32F => 8u8,
			TextureFormat::R32F => 4u8,
			TextureFormat::Depth16 => 2u8,
			TextureFormat::Depth24 => 4u8, // Although the internal format is 24-bit, write/readback uses 32-bit values
			TextureFormat::Depth32F => 4u8,
			TextureFormat::Depth24Stencil8 => 4u8,
		}) as usize
	}

	/// Returns true if the format is a depth format.
	#[inline]
	pub const fn is_depth(self) -> bool {
		matches!(self,
			| TextureFormat::Depth16
			| TextureFormat::Depth24
			| TextureFormat::Depth32F
			| TextureFormat::Depth24Stencil8
		)
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

/// Texture usage flags.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TextureUsage(u8);
impl TextureUsage {
	pub const NONE: TextureUsage = TextureUsage(0x0);
	/// Texture can be written to using [IGraphics::texture2d_write].
	pub const WRITE: TextureUsage = TextureUsage(0x1);
	/// Texture can be read back using [IGraphics::texture2d_read_into].
	pub const READBACK: TextureUsage = TextureUsage(0x2);
	/// Texture can be sampled in shaders.
	pub const SAMPLED: TextureUsage = TextureUsage(0x4);
	/// Texture can be used as a color render target.
	pub const COLOR_TARGET: TextureUsage = TextureUsage(0x8);
	/// Texture can be used as a depth/stencil render target.
	pub const DEPTH_STENCIL_TARGET: TextureUsage = TextureUsage(0x10);

	/// Common usage for textures used as sampled textures.
	pub const TEXTURE: TextureUsage = TextureUsage::WRITE.or(TextureUsage::SAMPLED);

	/// Combines usage flags.
	#[inline]
	pub const fn or(self, other: TextureUsage) -> TextureUsage {
		TextureUsage(self.0 | other.0)
	}

	/// Checks if all of the specified usage flags are set.
	#[inline]
	pub const fn all(self, other: TextureUsage) -> bool {
		(self.0 & other.0) == other.0
	}

	/// Checks if any of the specified usage flags are set.
	#[inline]
	pub const fn has(self, other: TextureUsage) -> bool {
		(self.0 & other.0) != 0
	}
}
impl Default for TextureUsage {
	#[inline]
	fn default() -> TextureUsage {
		TextureUsage::TEXTURE
	}
}
impl ops::BitOr for TextureUsage {
	type Output = TextureUsage;
	#[inline]
	fn bitor(self, rhs: TextureUsage) -> TextureUsage {
		TextureUsage(self.0 | rhs.0)
	}
}
/// Macro to create TextureUsage flags.
#[macro_export]
macro_rules! TextureUsage {
	($($flag:ident)|+ $(|)?) => {
		const { ::shade::TextureUsage::NONE $(.or(::shade::TextureUsage::$flag))+ }
	};
}

/// Texture properties.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextureProps {
	pub mip_levels: u8,
	pub usage: TextureUsage,
	pub filter_min: TextureFilter,
	pub filter_mag: TextureFilter,
	pub wrap_u: TextureWrap,
	pub wrap_v: TextureWrap,
	pub compare: Option<Compare>,
	pub border_color: [f32; 4],
}

impl Default for TextureProps {
	#[inline]
	fn default() -> TextureProps {
		TextureProps {
			mip_levels: 1,
			usage: TextureUsage::TEXTURE,
			filter_min: TextureFilter::Linear,
			filter_mag: TextureFilter::Linear,
			wrap_u: TextureWrap::Edge,
			wrap_v: TextureWrap::Edge,
			compare: None,
			border_color: [0.0, 0.0, 0.0, 0.0],
		}
	}
}

//----------------------------------------------------------------
// Texture2D handle.

define_handle!(Texture2D);

/// Texture2D information.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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

/// Load various image types into textures.
pub trait ImageToTexture {
	/// Get the texture info of the image.
	fn info(&self) -> Texture2DInfo;
	/// Get the raw pixel data of the image.
	fn data(&self) -> &[u8];
}
