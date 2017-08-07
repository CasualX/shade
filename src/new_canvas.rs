
use ::std::ops;
use super::{Primitive, Index, IVertex, Shader};

pub trait VertexBuffer<V> {
	fn alloc(&mut self, n: usize) -> &mut [V];
}

pub trait ICanvas {
	type VertexBuffers;
	fn shade<'a, S: Shader<'a>>(&'a mut self) -> S where Self::VertexBuffers: VertexBuffer<S::Vertex>;
}

#[derive(Clone, Debug)]
pub struct Batch {
	vertex_uid: u32,
	shader_uid: u32,
	vertices: ops::Range<u32>,
	indices: ops::Range<u32>,
}

pub struct CanvasLock<'a, VB: 'a>(&'a mut Canvas<VB>);
impl<'a, VB> CanvasLock<'a, VB> {
	pub fn draw_primitive<V: IVertex>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [V], &mut [Index]) where VB: VertexBuffer<V> {
		// Allocate indices
		let nindices = prim as usize * nprims;
		if self.0.indices.capacity() - self.0.indices.len() < nindices {
			let additional = nindices - (self.0.indices.capacity() - self.0.indices.len());
			self.0.indices.reserve(additional);
		}
		let indices = {
			let start = self.0.indices.len();
			unsafe { self.0.indices.set_len(start + nindices); }
			&mut self.0.indices[start..]
		};
		// Initialize indices
		for i in indices.iter_mut() {
			*i = self.0.istart;
		}
		self.0.istart = self.0.istart.checked_add(nindices as Index).expect("indices overflow");
		// Allocate vertices
		let verts = self.0.verts.alloc(nverts);
		(verts, indices)
	}
}

#[derive(Clone, Debug, Default)]
pub struct Canvas<VB> {
	verts: VB,
	indices: Vec<Index>,
	batches: Vec<Batch>,
	istart: Index,
}
impl<VB> ICanvas for Canvas<VB> {
	type VertexBuffers = VB;
	fn shade<'a, S>(&'a mut self) -> S where
		Self: Sized,
		Self::VertexBuffers: VertexBuffer<S::Vertex>,
		S: Shader<'a>
	{
		S::new(CanvasLock(self))
	}
}
