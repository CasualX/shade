
use {TShader, TUniform};
use d2::ColorV;
use super::Uniforms;

#[derive(Copy, Clone, Debug)]
pub struct Solid {}

impl TUniform for Solid {
	fn uniform_uid() -> u32 { 0x116e33ac }
}
impl TShader for Solid {
	type Vertex = ColorV;
	type Uniform = ();

	fn shader_uid() -> u32 { 0x128facde }
}
