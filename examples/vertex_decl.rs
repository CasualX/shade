extern crate shade;

use shade::d2::ColorV;
use shade::{ICanvas, Shader};

mod vb {
	use shade::{VertexBuffer};
	use shade::d2::{ColorV, TexV};
	#[derive(Clone, Default)]
	pub struct VertexBuffers {
		colorv: Vec<ColorV>,
		texv: Vec<TexV>,
	}
	impl VertexBuffer<ColorV> for VertexBuffers {
		fn allocate(&mut self, n: usize) -> &mut [ColorV] {
			let start = self.colorv.len();
			self.colorv.resize(start + n, ColorV::default());
			&mut self.colorv[start..]
		}
	}
	impl VertexBuffer<TexV> for VertexBuffers {
		fn allocate(&mut self, n: usize) -> &mut [TexV] {
			let start = self.texv.len();
			self.texv.resize(start + n, TexV::default());
			&mut self.texv[start..]
		}
	}
}

// struct MyShader;
// impl Shader for MyShader {
// }

fn main() {
	let mut canvas = shade::Canvas::<vb::VertexBuffers>::default();

	// let (verts, indices) = canvas.draw_primitive::<ColorV>(shade::Primitive::Lines, 4, 4);
}
