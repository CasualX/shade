
pub trait UniformData {
	fn uid() -> u32;
}

impl UniformData for () {
	fn uid() -> u32 { 0x9d5512f1 }
}

pub trait UniformBuffer<U: UniformData> {
	fn allocate(&mut self, n: usize) -> &mut [U];
}
