
/// Pixel format trait.
pub trait PixelFormat {
	const FORMAT: crate::TextureFormat;
	const CHANNELS: usize;
}

/// 8-bit linear RGB color with alpha.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(C)]
pub struct Srgba8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}

unsafe impl dataview::Pod for Srgba8 {}

impl PixelFormat for Srgba8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::SRGBA8;
	const CHANNELS: usize = 4;
}

/// 8-bit linear RGB color.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(C)]
pub struct Srgb8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl From<[u8; 3]> for Srgb8 {
	#[inline]
	fn from([r, g, b]: [u8; 3]) -> Srgb8 {
		Srgb8 { r, g, b }
	}
}

unsafe impl dataview::Pod for Srgb8 {}

impl PixelFormat for Srgb8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::SRGB8;
	const CHANNELS: usize = 3;
}

/// 8-bit linear RGB color with alpha.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(C)]
pub struct Rgba8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}

unsafe impl dataview::Pod for Rgba8 {}

impl PixelFormat for Rgba8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RGBA8;
	const CHANNELS: usize = 4;
}

/// 8-bit linear RGB color.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(C)]
pub struct Rgb8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

unsafe impl dataview::Pod for Rgb8 {}

impl PixelFormat for Rgb8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RGB8;
	const CHANNELS: usize = 3;
}

/// 8-bit two-channel color.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(C)]
pub struct Rg8 {
	pub r: u8,
	pub g: u8,
}

unsafe impl dataview::Pod for Rg8 {}

impl PixelFormat for Rg8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RG8;
	const CHANNELS: usize = 2;
}

/// 8-bit single channel color.
#[derive(Copy, Clone, Debug, Default, PartialEq, Hash)]
#[repr(transparent)]
pub struct R8(pub u8);

unsafe impl dataview::Pod for R8 {}

impl PixelFormat for R8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::R8;
	const CHANNELS: usize = 1;
}

/// Linear 32-bit float linear RGB color with alpha.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Rgba32f {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

unsafe impl dataview::Pod for Rgba32f {}

impl PixelFormat for Rgba32f {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RGBA32F;
	const CHANNELS: usize = 4;
}

/// Linear 32-bit float linear RGB color.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Rgb32f {
	pub r: f32,
	pub g: f32,
	pub b: f32,
}

unsafe impl dataview::Pod for Rgb32f {}

impl PixelFormat for Rgb32f {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RGB32F;
	const CHANNELS: usize = 3;
}

/// Linear 32-bit float two-channel color.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Rg32f {
	pub r: f32,
	pub g: f32,
}

unsafe impl dataview::Pod for Rg32f {}

impl PixelFormat for Rg32f {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::RG32F;
	const CHANNELS: usize = 2;
}

/// Linear 32-bit float single channel color.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(transparent)]
pub struct R32f(pub f32);

unsafe impl dataview::Pod for R32f {}

impl PixelFormat for R32f {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::R32F;
	const CHANNELS: usize = 1;
}

/// 16-bit depth value.
#[repr(transparent)]
pub struct Depth16(pub u16);

unsafe impl dataview::Pod for Depth16 {}

impl PixelFormat for Depth16 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::Depth16;
	const CHANNELS: usize = 1;
}

/// 24-bit depth value stored in a u32.
#[repr(transparent)]
pub struct Depth24(pub u32);

unsafe impl dataview::Pod for Depth24 {}

impl PixelFormat for Depth24 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::Depth24;
	const CHANNELS: usize = 1;
}

/// 32-bit float depth value.
#[repr(transparent)]
pub struct Depth32f(pub f32);

unsafe impl dataview::Pod for Depth32f {}

impl PixelFormat for Depth32f {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::Depth32F;
	const CHANNELS: usize = 1;
}

/// 24-bit depth and 8-bit stencil packed into a u32.
#[repr(transparent)]
pub struct Depth24Stencil8(pub u32);
impl Depth24Stencil8 {
	#[inline]
	pub fn new(depth: u32, stencil: u8) -> Depth24Stencil8 {
		assert!(depth <= 0x00FF_FFFF);
		Depth24Stencil8((stencil as u32) << 24 | (depth & 0x00FF_FFFF))
	}
	#[inline]
	pub fn depth(&self) -> u32 {
		self.0 & 0x00FF_FFFF
	}
	#[inline]
	pub fn stencil(&self) -> u8 {
		(self.0 >> 24) as u8
	}
}

unsafe impl dataview::Pod for Depth24Stencil8 {}

impl PixelFormat for Depth24Stencil8 {
	const FORMAT: crate::TextureFormat = crate::TextureFormat::Depth24Stencil8;
	const CHANNELS: usize = 1;
}
