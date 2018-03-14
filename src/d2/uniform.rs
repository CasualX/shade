use {TUniform, Blend};
use super::{Affine2, Color};

#[derive(Copy, Clone, Debug, Default)]
pub struct Uniform {
	pub blend: Blend,
	pub xytransform: Affine2,
	pub uvtransform: Affine2,
	pub colormod: Color,
}

impl TUniform for Uniform {
	fn uniform_uid() -> u32 { 620182 }
}
