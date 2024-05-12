use std::mem;

use super::*;

pub(super) struct Command {
	prim_type: PrimType,
	blend_mode: BlendMode,
	scissor_test: Option<cvmath::Rect<i32>>,
	shader: Shader,
	vertex_start: u32,
	vertex_end: u32,
	index_start: u32,
	index_end: u32,
	uniform_index: u32,
}

/// Command buffer.
pub struct CommandBuffer<V, U> {
	pub(super) vertices: Vec<V>,
	pub(super) indices: Vec<u32>,
	pub(super) uniforms: Vec<U>,
	pub(super) commands: Vec<Command>,

	pub blend_mode: BlendMode,
	pub shader: Shader,
	pub viewport: Rect<i32>,
	pub scissor_test: Option<cvmath::Rect<i32>>,
	pub depth_test: Option<DepthTest>,
	pub cull_mode: Option<CullMode>,
}

impl<V: TVertex, U: TUniform> CommandBuffer<V, U> {
	/// Creates a new command buffer.
	pub fn new() -> Self {
		CommandBuffer {
			vertices: Vec::new(),
			indices: Vec::new(),
			uniforms: Vec::new(),
			commands: Vec::new(),

			blend_mode: BlendMode::Solid,
			shader: Shader::INVALID,
			viewport: Rect::ZERO,
			scissor_test: None,
			depth_test: None,
			cull_mode: None,
		}
	}

	/// Clears the command buffer for reuse.
	pub fn clear(&mut self) {
		self.vertices.clear();
		self.indices.clear();
		self.uniforms.clear();
		self.commands.clear();
		self.blend_mode = BlendMode::Solid;
		self.shader = Shader::INVALID;
		self.viewport = Rect::ZERO;
		self.scissor_test = None;
		self.depth_test = None;
		self.cull_mode = None;
	}

	/// Draws the command buffer.
	pub fn draw(&self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError> {
		let vb = g.vertex_buffer(None, &self.vertices, BufferUsage::Static)?;
		let ib = g.index_buffer(None, &self.indices, BufferUsage::Static)?;
		let ub = g.uniform_buffer(None, &self.uniforms)?;

		for cmd in &self.commands {
			g.draw_indexed(&DrawIndexedArgs {
				surface,
				viewport: self.viewport,
				scissor: self.scissor_test,
				blend_mode: cmd.blend_mode,
				depth_test: self.depth_test,
				cull_mode: self.cull_mode,
				prim_type: cmd.prim_type,
				shader: cmd.shader,
				vertices: vb,
				indices: ib,
				uniforms: ub,
				vertex_start: cmd.vertex_start,
				vertex_end: cmd.vertex_end,
				index_start: cmd.index_start,
				index_end: cmd.index_end,
				uniform_index: cmd.uniform_index,
				instances: -1,
			})?;
		}

		g.uniform_buffer_delete(ub, true)?;
		g.index_buffer_delete(ib, true)?;
		g.vertex_buffer_delete(vb, true)?;
		Ok(())
	}

	/// Gets the current uniform.
	pub fn get_uniform(&mut self) -> &mut U {
		if self.uniforms.is_empty() {
			self.uniforms.push(U::default());
		}
		self.uniforms.last_mut().unwrap()
	}

	/// Pushes a new uniform to the command buffer.
	pub fn push_uniform(&mut self, u: U) {
		self.uniforms.push(u);
	}

	/// Pushes a new uniform to the command buffer derived from the last uniforms.
	pub fn push_uniform_f<F: FnOnce(&U) -> U>(&mut self, f: F) {
		if self.uniforms.is_empty() {
			self.uniforms.push(U::default());
		}
		let u = f(self.uniforms.last().unwrap());
		self.uniforms.push(u);
	}

	/// Begins adding a new command to the command buffer.
	pub fn begin<'a>(&'a mut self, prim_type: PrimType, nverts: usize, nprims: usize) -> PrimBuilder<'a, V> {
		// Ensure there is at least one uniform.
		if self.uniforms.is_empty() {
			self.uniforms.push(U::default());
		}

		let nindices = nprims * match prim_type { PrimType::Triangles => 3, PrimType::Lines => 2, };

		// Check if the new command can be merged with the last command.
		let mut new_cmd = true;
		if let Some(last) = self.commands.last_mut() {
			let compatible =
				last.shader == self.shader &&
				last.prim_type == prim_type &&
				last.blend_mode == self.blend_mode &&
				last.scissor_test == self.scissor_test &&
				last.uniform_index + 1 == self.uniforms.len() as u32;
			if compatible {
				last.vertex_end += nverts as u32;
				last.index_end += nindices as u32;
				new_cmd = false;
			}
		}

		// Otherwise add a new command.
		if new_cmd {
			let blend_mode = self.blend_mode;
			let scissor_test = self.scissor_test;
			let shader = self.shader;
			let vertex_start = self.vertices.len() as u32;
			let vertex_end = vertex_start + nverts as u32;
			let index_start = self.indices.len() as u32;
			let index_end = index_start + nindices as u32;
			let uniform_index = self.uniforms.len() as u32 - 1;
			self.commands.push(Command { prim_type, blend_mode, scissor_test, shader, vertex_start, vertex_end, index_start, index_end, uniform_index });
		}

		let vertex_start = self.vertices.len();
		let index_start = self.indices.len();

		// Reserve space for new data.
		self.vertices.resize_with(vertex_start + nverts, V::default);
		self.indices.resize(index_start + nindices, vertex_start as u32);

		PrimBuilder {
			vertices: &mut self.vertices[vertex_start..],
			indices: &mut self.indices[index_start..],
			#[cfg(debug_assertions)]
			nvertices: nverts,
		}
	}
}

/// Builder for adding vertices and indices to a command buffer.
pub struct PrimBuilder<'a, V: TVertex> {
	vertices: &'a mut [V],
	indices: &'a mut [u32],
	#[cfg(debug_assertions)]
	nvertices: usize,
}

impl<'a, V: TVertex> PrimBuilder<'a, V> {
	/// Adds a vertex index to the command buffer.
	#[track_caller]
	pub fn add_index(&mut self, vertex: u32) {
		#[cfg(debug_assertions)]
		{
			assert!(self.indices.len() >= 1, "too many indices");
			assert!((vertex as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(1);
		head[0] += vertex;
		self.indices = tail;
	}

	/// Adds two vertex indices to the command buffer.
	#[track_caller]
	pub fn add_index2(&mut self, vertex1: u32, vertex2: u32) {
		#[cfg(debug_assertions)]
		{
			assert!(self.indices.len() >= 2, "too many indices");
			assert!((vertex1 as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex1, self.nvertices);
			assert!((vertex2 as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex2, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(2);
		head[0] += vertex1;
		head[1] += vertex2;
		self.indices = tail;
	}

	/// Adds three vertex indices to the command buffer.
	#[track_caller]
	pub fn add_index3(&mut self, vertex1: u32, vertex2: u32, vertex3: u32) {
		#[cfg(debug_assertions)]
		{
			assert!(self.indices.len() >= 3, "too many indices");
			assert!((vertex1 as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex1, self.nvertices);
			assert!((vertex2 as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex2, self.nvertices);
			assert!((vertex3 as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", vertex3, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(3);
		head[0] += vertex1;
		head[1] += vertex2;
		head[2] += vertex3;
		self.indices = tail;
	}

	/// Adds vertex indices to the command buffer.
	#[track_caller]
	pub fn add_indices(&mut self, indices: &[u32]) {
		#[cfg(debug_assertions)]
		{
			assert!(self.indices.len() >= indices.len(), "too many indices");
			for &index in indices {
				assert!((index as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", index, self.nvertices);
			}
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(indices.len());
		for i in 0..indices.len() {
			head[i] += indices[i];
		}
		self.indices = tail;
	}

	/// Adds the vertex indices for a quad to the command buffer.
	///
	/// ```text
	/// 1---2
	/// | / |
	/// 0---3
	/// ```
	#[track_caller]
	pub fn add_indices_quad(&mut self) {
		self.add_indices(&[0, 1, 2, 0, 2, 3]);
	}

	/// Adds a vertex to the command buffer.
	#[track_caller]
	pub fn add_vertex(&mut self, vertex: V) {
		#[cfg(debug_assertions)]
		{
			assert!(self.vertices.len() >= 1, "too many vertices");
		}

		let (head, tail) = mem::replace(&mut self.vertices, &mut []).split_at_mut(1);
		head[0] = vertex;
		self.vertices = tail;
	}

	/// Adds vertices to the command buffer.
	#[track_caller]
	pub fn add_vertices(&mut self, vertices: &[V]) {
		#[cfg(debug_assertions)]
		{
			assert!(self.vertices.len() >= vertices.len(), "too many vertices");
		}

		let (head, tail) = mem::replace(&mut self.vertices, &mut []).split_at_mut(vertices.len());
		head.copy_from_slice(vertices);
		self.vertices = tail;
	}
}

#[cfg(debug_assertions)]
impl<'a, V: TVertex> Drop for PrimBuilder<'a, V> {
	#[track_caller]
	fn drop(&mut self) {
		assert!(self.indices.is_empty(), "expected more indices, {} left", self.indices.len());
		assert!(self.vertices.is_empty(), "expected more vertices, {} left", self.vertices.len());
	}
}
