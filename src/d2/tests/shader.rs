/*!
Mocked shader.
*/

use ::{Primitive, Index, Shader, IVertex};

#[derive(Debug, Default)]
pub struct MockShader<V> {
	pub prim: Primitive,
	pub istart: Index,
	pub nprims: usize,
	pub verts: Vec<V>,
	pub indices: Vec<Index>,
}

impl<V: IVertex> Shader<'static> for MockShader<V> {
	type Vertex = V;
	type Context = ();
	fn uid() -> u32 { 0 }
	fn context(&self) -> () {}
	fn set_context(&mut self, ctx: &()) {}
	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [Self::Vertex], &mut [Index]) {
		self.prim = prim;
		self.istart += 1 + self.verts.len() as Index;
		self.nprims = nprims;
		if self.verts.capacity() < nverts {
			let reserve = nverts - self.verts.capacity();
			self.verts.reserve(reserve);
		}
		let verts = unsafe {
			self.verts.set_len(nverts);
			&mut self.verts[..]
		};
		let nindices = prim as u8 as usize * nprims;
		if self.indices.capacity() < nindices {
			let reserve = nindices - self.indices.capacity();
			self.indices.reserve(reserve);
		}
		let indices = unsafe {
			self.indices.set_len(nindices);
			for it in &mut self.indices {
				*it = self.istart;
			}
			&mut self.indices[..]
		};
		(verts, indices)
	}
	fn new_batch(&mut self) {}
}
