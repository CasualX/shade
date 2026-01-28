//! Dither patterns.
#![allow(non_upper_case_globals)]

/// Dither pattern image.
///
/// Can be used directly as a texture:
///
/// ```rust
/// fn example(g: &mut shade::Graphics) {
/// 	let dither_texture = g.image(&shade::dither::BAYER4x4);
/// }
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Image<const N: usize> {
	pub width: u8,
	pub height: u8,
	pub data: [u8; N],
}

impl<const N: usize> crate::ImageToTexture for Image<N> {
	fn info(&self) -> crate::Texture2DInfo {
		crate::Texture2DInfo {
			width: self.width as i32,
			height: self.height as i32,
			format: crate::TextureFormat::R8,
			props: crate::TextureProps {
				mip_levels: 1,
				usage: crate::TextureUsage::TEXTURE,
				filter_min: crate::TextureFilter::Nearest,
				filter_mag: crate::TextureFilter::Nearest,
				wrap_u: crate::TextureWrap::Repeat,
				wrap_v: crate::TextureWrap::Repeat,
				..Default::default()
			},
		}
	}
	fn data(&self) -> &[u8] {
		&self.data
	}
}

/// Bayer 2x2 dither pattern.
pub static BAYER2x2: Image<4> = Image {
	width: 2,
	height: 2,
	data: *include_bytes!("dither/bayer2x2.bin"),
};

/// Bayer 4x4 dither pattern.
pub static BAYER4x4: Image<16> = Image {
	width: 4,
	height: 4,
	data: *include_bytes!("dither/bayer4x4.bin"),
};
/// Bayer 8x8 dither pattern.
pub static BAYER8x8: Image<64> = Image {
	width: 8,
	height: 8,
	data: *include_bytes!("dither/bayer8x8.bin"),
};
/// Bayer 16x16 dither pattern.
pub static BAYER16x16: Image<256> = Image {
	width: 16,
	height: 16,
	data: *include_bytes!("dither/bayer16x16.bin"),
};

/// Blue noise (voids and clusters) 32x32 dither pattern.
pub static BNVC32x32: Image<1024> = Image {
	width: 32,
	height: 32,
	data: *include_bytes!("dither/bnvc32x32.bin"),
};
/// Blue noise (voids and clusters) 64x64 dither pattern.
pub static BNVC64x64: Image<4096> = Image {
	width: 64,
	height: 64,
	data: *include_bytes!("dither/bnvc64x64.bin"),
};

const fn invert<const N: usize>(data: [u8; N]) -> [u8; N] {
	let mut out = [0u8; N];
	let mut i = 0;
	while i < N {
		out[i] = 255 - data[i];
		i += 1;
	}
	out
}

/// Halftone 16x16 dither pattern.
pub static HALFTONE16x16: Image<256> = Image {
	width: 16,
	height: 16,
	data: invert([
		253,245,239,224,212,203,186,177,175,181,206,215,229,236,250,254,
		248,233,217,194,165,151,143,131,132,145,150,167,192,220,232,246,
		242,222,188,156,129,117, 98, 91, 92, 96,113,135,162,189,218,241,
		228,201,157,121,109, 81, 69, 60, 61, 68, 84,104,122,160,196,225,
		209,166,126,106, 78, 59, 45, 32, 41, 48, 55, 79,103,134,171,211,
		199,153,118, 87, 57, 40, 25, 20, 17, 26, 36, 53, 82,116,148,195,
		180,144,102, 71, 51, 27, 13,  4,  6, 15, 24, 46, 73,101,141,183,
		176,130, 93, 67, 35, 18,  9,  2,  0,  7, 19, 33, 64, 94,127,172,
		174,125, 95, 63, 34, 16,  8,  1,  3, 10, 22, 38, 62, 90,137,173,
		182,142, 99, 72, 50, 28, 14, 11,  5, 12, 30, 44, 70,100,140,185,
		197,149,112, 85, 54, 39, 29, 21, 23, 31, 42, 58, 86,115,152,193,
		210,168,124, 97, 76, 56, 49, 43, 37, 47, 52, 77,107,128,169,214,
		226,202,159,123,110, 83, 74, 66, 65, 75, 80,108,120,163,198,227,
		243,223,190,161,136,119,105, 89, 88,111,114,133,158,191,216,237,
		244,234,219,200,164,155,146,138,139,147,154,170,205,221,235,247,
		252,249,240,230,213,204,184,178,179,187,207,208,231,238,251,255,
	]),
};
