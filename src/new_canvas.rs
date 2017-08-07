
use ::std::ops;
use super::{Primitive, Index, IVertex};

pub trait VertexBuffer<V> {
	fn alloc(&mut self, n: usize) -> &mut [V];
}

pub trait ICanvas {
	type VB;
	fn draw_primitive<V: IVertex>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [V], &mut [Index]) where Self::VB: VertexBuffer<V>;
	fn new_batch(&mut self);
}

#[derive(Clone, Debug)]
pub struct Batch {
	vertex_uid: u32,
	shader_uid: u32,
	vertices: ops::Range<u32>,
	indices: ops::Range<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct Canvas<VB> {
	verts: VB,
	indices: Vec<Index>,
	batches: Vec<Batch>,
	istart: Index,
}
impl<VB> ICanvas for Canvas<VB> {
	type VB = VB;
	fn draw_primitive<V: IVertex>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [V], &mut [Index]) where VB: VertexBuffer<V> {
		// Allocate indices
		let nindices = prim as usize * nprims;
		if self.indices.capacity() - self.indices.len() < nindices {
			let additional = nindices - (self.indices.capacity() - self.indices.len());
			self.indices.reserve(additional);
		}
		let indices = {
			let start = self.indices.len();
			unsafe { self.indices.set_len(start + nindices); }
			&mut self.indices[start..]
		};
		// Initialize indices
		for i in indices.iter_mut() {
			*i = self.istart;
		}
		self.istart = self.istart.checked_add(nindices as Index).expect("indices overflow");
		// Allocate vertices
		let verts = self.verts.alloc(nverts);
		(verts, indices)
	}
	fn new_batch(&mut self) {
		unimplemented!()
	}
}
