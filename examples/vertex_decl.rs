extern crate shade;

use shade::d2::*;
use shade::*;
use shade::d2::Canvas;

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
	let pen = Pen {
		color: Color::new(0.75, 0.25, 0.0, 1.0),
		shader: MyShader,
		..Pen::default()
	};
	let rc = Rect::new(Point2::new(1.0, 2.0), Point2::new(10.0, 20.0));
	cv.fill_rect(&paint, &rc);
	cv.fill_rect(&paint, &rc);
	cv.draw_line_rect(&pen, &rc);
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
		if batch.prim == Primitive::Lines && batch.shader_uid == MyShader::shader_uid() && batch.vertex_uid == ColorV::vertex_uid() {
			render_lines_myshader_colorv(
				&cv.buffers.colorv[..batch.nverts as usize],
				&cv.indices[..(batch.nprims * 3) as usize],
			);
		}
	}
}

fn render_triangles_myshader_colorv(vertices: &[ColorV], indices: &[Index]) {
	println!("render triangles: {} vertices and {} indices: {:?}", vertices.len(), indices.len(), indices);
}
fn render_lines_myshader_colorv(vertices: &[ColorV], indices: &[Index]) {
	println!("render lines: {} vertices and {} indices: {:?}", vertices.len(), indices.len(), indices);
}
