
use super::{Primitive, Index, TVertex, Allocate, TShader, TUniform};

pub trait ICanvas {
	type Buffers;
	fn draw_primitive<S>(&mut self, prim: Primitive, nverts: usize, nprims: usize)
		-> (&mut [S::Vertex], &mut [Index]) where S: TShader, Self::Buffers: Allocate<S::Vertex>;
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
pub struct Canvas<T> {
	buffers: T,
	indices: Vec<Index>,
	batches: Vec<Batch>,
	istart: Index,
}
impl<T> Canvas<T> {
	pub fn context<U>(&mut self) -> U where U: TUniform, T: Allocate<U> {
		unimplemented!()
	}
	pub fn set_context<U>(&mut self, _ctx: &U) where U: TUniform, T: Allocate<U> {
		unimplemented!()
	}
	pub fn pop_context<U>(&mut self) where U: TUniform, T: Allocate<U> {
		unimplemented!()
	}
	pub fn draw_primitive<S>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [S::Vertex], &mut [Index])
		where S: TShader, T: Allocate<S::Vertex>
	{
		let shader_uid = S::shader_uid();
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
				vertex_uid: S::Vertex::vertex_uid(),
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
		let verts = unsafe { self.buffers.allocate(nverts) };
		(verts, indices)
	}
}

impl<T> ICanvas for Canvas<T> {
	type Buffers = T;
	fn draw_primitive<S>(&mut self, prim: Primitive, nverts: usize, nprims: usize)
		-> (&mut [S::Vertex], &mut [Index]) where S: TShader, Self::Buffers: Allocate<S::Vertex>
	{
		Self::draw_primitive::<S>(self, prim, nverts, nprims)
	}
}
