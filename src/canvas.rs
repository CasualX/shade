
use super::{Primitive, Index, IVertex, VertexBuffer, Shader};

pub trait ICanvas {
	type VertexBuffers;
	fn draw_primitive<S>(&mut self, prim: Primitive, nverts: usize, nprims: usize)
		-> (&mut [S::Vertex], &mut [Index]) where S: Shader, Self::VertexBuffers: VertexBuffer<S::Vertex>;
}

#[derive(Copy, Clone, Debug)]
pub struct Batch {
	shader_uid: u32,
	vertex_uid: u32,
	prim: Primitive,
	nverts: Index,
	nprims: Index,
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
	fn draw_primitive<S>(&mut self, prim: Primitive, nverts: usize, nprims: usize)
		-> (&mut [S::Vertex], &mut [Index]) where S: Shader, Self::VertexBuffers: VertexBuffer<S::Vertex>
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
				else if last_batch.shader_uid != shader_uid || last_batch.prim != prim {
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
				prim: prim,
				nverts: 0,
				nprims: 0,
			});
		}
		let batch = self.batches.last_mut().unwrap();
		batch.prim = prim;
		batch.nprims += nprims as Index;
		batch.nverts += nverts as Index;
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
		let verts = self.verts.allocate(nverts);
		(verts, indices)
	}
}
