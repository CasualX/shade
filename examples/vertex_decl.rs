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
		fn allocate(&mut self, n: usize) -> &mut [ColorV] {
			let start = self.colorv.len();
			self.colorv.resize(start + n, ColorV::default());
			&mut self.colorv[start..]
		}
	}
	impl Allocate<TexV> for Buffers {
		fn allocate(&mut self, n: usize) -> &mut [TexV] {
			let start = self.texv.len();
			self.texv.resize(start + n, TexV::default());
			&mut self.texv[start..]
		}
	}
}

#[derive(Copy, Clone, Debug, Default)]
struct MyShader;
impl TUniform for MyShader {
	fn uid() -> u32 { 12345445 }
}
impl TShader for MyShader {
	type Vertex = ColorV;
	type Uniform = ();
}

fn main() {
	let mut cv = Canvas::<vb::Buffers>::default();
	let (verts, indices) = cv.draw_primitive::<MyShader>(Primitive::Lines, 4, 4);
}
