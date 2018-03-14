extern crate shade;

use shade::d2::ColorV;
use shade::*;

mod vb {
	use shade::{Allocate};
	use shade::d2::{ColorV, TexV};
	#[derive(Clone, Default)]
	pub struct Buffers {
		colorv: Vec<ColorV>,
		texv: Vec<TexV>,
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
}

#[derive(Copy, Clone, Debug, Default)]
struct MyShader;
impl TUniform for MyShader {
	fn uniform_uid() -> u32 { 12345445 }
}
impl TShader for MyShader {
	type Vertex = ColorV;
	type Uniform = ();
	fn shader_uid() -> u32 { 520192 }
}

fn main() {
	let mut cv = Canvas::<vb::Buffers>::default();
	let (verts, indices) = cv.draw_primitive::<MyShader>(Primitive::Lines, 4, 4);
}
