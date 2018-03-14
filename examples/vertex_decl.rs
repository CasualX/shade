extern crate shade;

use shade::d2::*;
use shade::*;

mod vb {
	use shade::{Allocate};
	use shade::d2::{ColorV, TexV};
	#[derive(Clone, Default)]
	pub struct Buffers {
		pub colorv: Vec<ColorV>,
		pub texv: Vec<TexV>,
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

type Canvas = shade::Canvas<vb::Buffers>;

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
	let mut cv = Canvas::default();
	let paint = Paint {
		color1: Color::new(0.5, 0.5, 0.5, 1.0),
		shader: MyShader,
		..Paint::default()
	};
	let rc = Rect::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0));
	cv.fill_rect(&paint, &rc);
	render(&cv);
}

fn render(cv: &Canvas) {
	for (index, batch) in cv.batches.iter().enumerate() {
		if batch.prim == Primitive::Triangles && batch.shader_uid == MyShader::shader_uid() && batch.vertex_uid == ColorV::vertex_uid() {
			render_triangles_myshader_colorv(
				&cv.buffers.colorv[..batch.nverts as usize],
				&cv.indices[..(batch.nprims * 3) as usize],
			);
		}
	}
}

fn render_triangles_myshader_colorv(vertices: &[ColorV], indices: &[Index]) {
	unimplemented!()
}
