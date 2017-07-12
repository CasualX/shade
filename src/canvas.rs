/*!
Canvas render queue batching.
*/

use ::{Index, PlaceV, Primitive};

pub trait ICanvas {
	unsafe fn draw_primitive(&mut self, shader_uid: u32, prim: Primitive, nverts: usize, nprims: usize) -> (*mut PlaceV, *mut Index);
	unsafe fn new_batch(&mut self);
}

#[derive(Debug)]
struct Batch {
	prim: Primitive,
	numvert: Index,
	numprim: Index,
	shader_uid: u32,
}

#[derive(Debug)]
struct VertexFormat {
	vert_uid: u32,
	size_of: u32,
}

#[derive(Debug)]
struct ContextFormat {
	ctx_uid: u32,
	size_of: u32,
}

#[derive(Debug)]
struct ShaderFormat {
	shader_uid: u32,
	vert_uid: u32,
	ctx_uid: u32,
}

#[derive(Debug)]
struct Format {
	vert: Vec<VertexFormat>,
	shader: Vec<ShaderFormat>,
	ctx: Vec<ContextFormat>,
}

#[derive(Debug)]
pub struct Canvas {
	formats: Format,
	verts: Vec<u8>,
	indices: Vec<Index>,
	batches: Vec<Batch>,
	ctxs: Vec<u8>,
}

impl ICanvas for Canvas {
	unsafe fn draw_primitive(&mut self, shader_uid: u32, prim: Primitive, nverts: usize, nprims: usize) -> (*mut PlaceV, *mut Index) {
		unimplemented!()
	}
	unsafe fn new_batch(&mut self) {
		unimplemented!()
	}
}
