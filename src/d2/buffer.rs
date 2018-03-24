use super::{ColorV, TexV, TextV, Uniform};
use Allocate;

/// D2 Buffers
#[derive(Clone, Default, Debug)]
pub struct Buffers {
	pub colorv: Vec<ColorV>,
	pub texv: Vec<TexV>,
	pub textv: Vec<TextV>,
	pub uniform: Vec<Uniform>,
}

impl Allocate<()> for Buffers {
	unsafe fn allocate(&mut self, _n: usize) -> &mut [()] {
		&mut []
	}
}
impl Allocate<ColorV> for Buffers {
	unsafe fn allocate(&mut self, n: usize) -> &mut [ColorV] {
		self.colorv.allocate(n)
	}
}
impl Allocate<TexV> for Buffers {
	unsafe fn allocate(&mut self, n: usize) -> &mut [TexV] {
		self.texv.allocate(n)
	}
}
impl Allocate<TextV> for Buffers {
	unsafe fn allocate(&mut self, n: usize) -> &mut [TextV] {
		self.textv.allocate(n)
	}
}
impl Allocate<Uniform> for Buffers {
	unsafe fn allocate(&mut self, n: usize) -> &mut [Uniform] {
		self.uniform.allocate(n)
	}
}
