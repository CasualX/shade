
use {Blend, TShader, TUniform};
use super::{Affine2, Color, ColorV, Rect, TexV};

#[derive(Copy, Clone, Debug, Default)]
pub struct Uniform {
	pub blend: Blend,
	pub xytransform: Affine2,
	pub uvtransform: Affine2,
	pub uvclip: Rect,
	pub colormod: Color,
}
impl TUniform for Uniform {
	fn uniform_uid() -> u32 { 620182 }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Solid;
impl TUniform for Solid {
	fn uniform_uid() -> u32 { 0x116e33ac }
}
impl TShader for Solid {
	type Vertex = ColorV;
	type Uniform = Uniform;
	fn shader_uid() -> u32 { 0x128facde }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Textured;
impl TUniform for Textured {
	fn uniform_uid() -> u32 { 0x8271721 }
}
impl TShader for Textured {
	type Vertex = TexV;
	type Uniform = Uniform;
	fn shader_uid() -> u32 { 0xce92ec83 }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SpriteBrush;
impl TUniform for SpriteBrush {
	fn uniform_uid() -> u32 { 0x05efd891 }
}
impl TShader for SpriteBrush {
	type Vertex = TexV;
	type Uniform = Uniform;
	fn shader_uid() -> u32 { 0x2436a7dc }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MaskBrush;
impl TUniform for MaskBrush {
	fn uniform_uid() -> u32 { 0x758f48bc }
}
impl TShader for MaskBrush {
	type Vertex = TexV;
	type Uniform = Uniform;
	fn shader_uid() -> u32 { 0xfb038e65 }
}
