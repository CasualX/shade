
use {ICanvas, Allocate, Primitive, Index, TShader, TVertex};

/// Mocked canvas.
#[derive(Debug, Default)]
pub struct MockCanvas<V> {
	pub prim: Primitive,
	pub istart: Index,
	pub nprims: usize,
	pub verts: Vec<V>,
	pub indices: Vec<Index>,
}

impl<V: TVertex> ICanvas for MockCanvas<V> {
	type Buffers = Vec<V>;
	fn draw_primitive<S: TShader>(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [S::Vertex], &mut [Index])
		where Self::Buffers: Allocate<S::Vertex>
	{
		self.prim = prim;
		self.nprims = nprims;
		// Allocate indices
		self.istart += 1 + self.verts.len() as Index;
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
		// Allocate vertices
		let verts = self.verts.allocate(nverts);
		(verts, indices)
	}
}
