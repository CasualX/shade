
use cgmath::{Affine2, Vec4};

#[derive(Copy, Clone, Debug, Default)]
pub struct Uniforms {
	pub xytransform: Affine2<f32>,
	pub uvtransform: Affine2<f32>,
	pub colormod: Vec4<f32>,
}
