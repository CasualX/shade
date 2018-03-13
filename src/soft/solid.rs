
use {TShader, TUniform};
use d2::ColorV;
use super::Uniforms;

#[derive(Copy, Clone, Debug)]
pub struct Solid {}

impl TUniform for Solid {
	fn uid() -> u32 { 0x116e33ac }
}
impl TShader for Solid {
	type Vertex = ColorV;
	type Uniform = ();
}
