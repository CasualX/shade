/*!
*/

pub trait TUniform: Copy + 'static {
	fn uid() -> u32;
}

impl TUniform for () {
	fn uid() -> u32 { 0x9d5512f1 }
}
