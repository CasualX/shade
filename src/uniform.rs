/*!
*/

pub trait TUniform: Copy + 'static {
	fn uniform_uid() -> u32;
}

impl TUniform for () {
	fn uniform_uid() -> u32 { 0x9d5512f1 }
}
