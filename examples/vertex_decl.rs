extern crate shade;

use shade::d2::ColorV;
use shade::*;

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

struct MyShader<'a> {
	canvas: &'a mut Canvas::<vb::VertexBuffers>,
}
impl<'a> Shader for MyShader<'a> {
	type Vertex = ColorV;
	type Context = ();
	fn uid() -> u32 { 123 }
	fn context(&self) -> () { }
	fn set_context(&mut self, _ctx: &()) {}

	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [Self::Vertex], &mut [Index]) {
		self.canvas.draw_primitive::<Self>(prim, nverts, nprims)
	}
}

fn main() {
	let mut canvas = Canvas::<vb::VertexBuffers>::default();
	let mut shader = MyShader { canvas: &mut canvas };

	let (verts, indices) = shader.draw_primitive(Primitive::Lines, 4, 4);
}
