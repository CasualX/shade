
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
	shader_uid: u32,
	vertex_uid: u32,
	prim: Primitive,
	num_vert: Index,
	num_prim: Index,
}

pub struct CanvasLock<'a, VB: 'a>(&'a mut Canvas<VB>);
impl<'a, VB> CanvasLock<'a, VB> {
	pub fn draw_primitive<V: IVertex>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [V], &mut [Index]) where VB: VertexBuffer<V> {
		let batch = {
			let last_batch = *self.0.batches.last_mut().expect("expected at least one batch");
			if last_batch.prim == Primitive::Unspecified {
				self.0.batches.last_mut().unwrap()
			}
			else {
				self.0.batches.push(Batch { prim, num_vert: 0, num_prim: 0, ..last_batch });
				self.0.batches.last_mut().unwrap()
			}
		};
		batch.prim = prim;
		batch.num_prim += nprims as Index;
		batch.num_vert += nverts as Index;
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
		let shader_uid = S::uid();
		let new_batch = {
			if let Some(last_batch) = self.batches.last_mut() {
				// If the last batch has not been specialized, claim it
				if last_batch.shader_uid == 0 {
					last_batch.shader_uid = shader_uid;
					false
				}
				// If last batch has a different primitive or shader_uid, new batch
				else if last_batch.shader_uid != shader_uid {
					true
				}
				// Appending to the last batch
				else {
					false
				}
			}
			// Create the first batch
			else {
				true
			}
		};
		if new_batch {
			self.batches.push(Batch {
				vertex_uid: S::Vertex::uid(),
				shader_uid: shader_uid,
				vertices: 0..0,
				indices: 0..0,
			});
		}
		S::new(CanvasLock(self))
	}
}
