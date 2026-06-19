use std::collections::HashMap;
use std::any;

use super::*;

pub struct SharedState {
	pub scissor: Option<Bounds2<i32>>,
	pub blend_mode: BlendMode,
	pub depth_test: Option<Compare>,
	pub cull_mode: Option<CullMode>,
}

struct DrawData {
	vertices: Box<dyn VertexBuffer>,
	indices: Box<dyn IndexBuffer>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct BufferTypeId {
	vertex: any::TypeId,
	uniform: any::TypeId,
}

#[doc(hidden)]
pub trait BufferTypeKey {
	fn key() -> any::TypeId;
}

impl<'a> BufferTypeKey for d2::ColorUniform<'a> {
	#[inline]
	fn key() -> any::TypeId {
		any::TypeId::of::<d2::ColorUniform<'static>>()
	}
}

impl<'a> BufferTypeKey for d2::TextUniform<'a> {
	#[inline]
	fn key() -> any::TypeId {
		any::TypeId::of::<d2::TextUniform<'static>>()
	}
}

impl<'a> BufferTypeKey for d2::TextUniform3<'a> {
	#[inline]
	fn key() -> any::TypeId {
		any::TypeId::of::<d2::TextUniform3<'static>>()
	}
}

trait IDrawBuffer<'a> {
	fn buffer_type_id(&self) -> BufferTypeId;
	fn buffer_ptr(&mut self) -> *mut ();
	fn clear(&mut self);
	fn data(&self, g: &mut Graphics) -> DrawData;
	fn draw(&self, g: &mut Graphics);
	fn draw_range(&self, g: &mut Graphics, range: ops::Range<usize>, data: &DrawData);
	fn commands_len(&self) -> usize;
	fn shared_state(&self) -> SharedState;
}

impl<'a> dyn IDrawBuffer<'a> + 'a {
	#[inline]
	fn downcast_mut<V: TVertex, U: TUniform + BufferTypeKey + 'a>(&mut self) -> Option<&mut DrawBuilder<'a, V, U>> {
		if self.buffer_type_id() == (BufferTypeId { vertex: any::TypeId::of::<V>(), uniform: U::key() }) {
			Some(unsafe { &mut *(self.buffer_ptr() as *mut DrawBuilder<'a, V, U>) })
		}
		else {
			None
		}
	}
}

impl<'a, T: TVertex, U: TUniform + BufferTypeKey + 'a> IDrawBuffer<'a> for DrawBuilder<'a, T, U> {
	fn buffer_type_id(&self) -> BufferTypeId {
		BufferTypeId {
			vertex: any::TypeId::of::<T>(),
			uniform: U::key(),
		}
	}
	fn buffer_ptr(&mut self) -> *mut () {
		self as *mut Self as *mut ()
	}
	fn clear(&mut self) {
		self.clear();
	}
	fn data(&self, g: &mut Graphics) -> DrawData {
		let vertices = g.vertex_buffer(&self.vertices, BufferUsage::Static);
		let indices = g.index_buffer(&self.indices, self.vertices.len() as u16, BufferUsage::Static);
		DrawData { vertices, indices }
	}
	fn draw(&self, g: &mut Graphics) {
		draw(self, g)
	}
	fn draw_range(&self, g: &mut Graphics, range: ops::Range<usize>, data: &DrawData) {
		let batch = DrawBatch {
			commands: &self.commands[range],
			vertices: &*data.vertices,
			indices: &*data.indices,
		};
		draw_range(self, g, &batch)
	}
	fn commands_len(&self) -> usize {
		self.commands.len()
	}
	fn shared_state(&self) -> SharedState {
		SharedState {
			scissor: self.scissor,
			blend_mode: self.blend_mode,
			depth_test: self.depth_test,
			cull_mode: self.cull_mode,
		}
	}
}


pub struct DrawBatch<'a, 'r> {
	pub commands: &'a [drawbuf::DrawCommand<'r>],
	pub vertices: &'a dyn VertexBuffer,
	pub indices: &'a dyn IndexBuffer,
}

/// Draws the DrawBuilder.
fn draw<'a, V: TVertex, U: TUniform>(this: &DrawBuilder<'a, V, U>, g: &mut Graphics) {
	let vertices = g.vertex_buffer(&this.vertices, BufferUsage::Static);
	let indices = g.index_buffer(&this.indices, this.vertices.len() as u16, BufferUsage::Static);
	let range = DrawBatch {
		commands: &this.commands,
		vertices: &*vertices,
		indices: &*indices,
	};

	draw_range(this, g, &range);
}

/// Draws the specified commands from the buffer.
fn draw_range<'r, V: TVertex, U: TUniform>(this: &DrawBuilder<'r, V, U>, g: &mut Graphics, batch: &DrawBatch<'_, 'r>) {
	for cmd in batch.commands {
		let uniforms = &this.uniforms[cmd.pipeline_state.uniform_index as usize];
		g.draw_indexed(&DrawIndexedArgs {
			scissor: cmd.pipeline_state.scissor,
			blend_mode: cmd.pipeline_state.blend_mode,
			depth_test: cmd.pipeline_state.depth_test,
			cull_mode: cmd.pipeline_state.cull_mode,
			mask: cmd.pipeline_state.mask,
			prim_type: cmd.pipeline_state.prim_type,
			shader: cmd.pipeline_state.shader.expect("Draw command has no shader"),
			uniforms: &[uniforms],
			vertices: &[DrawVertexBuffer {
				buffer: batch.vertices,
				divisor: VertexDivisor::PerVertex,
			}],
			indices: batch.indices,
			index_start: cmd.index_start,
			index_end: cmd.index_end,
			instances: -1,
		});
	}
}


struct DrawSub {
	id: BufferTypeId,
	range: ops::Range<usize>,
}

/// Draw buffer pool.
///
/// Manages a collection of [DrawBuilder] with heterogeneous vertex and uniform types,
/// enabling efficient reuse and batching of draw commands.
///
/// Use [get](Self::get) to obtain a compatible `DrawBuilder<T, U>` for adding geometry.
/// Buffers are reused when possible to minimize allocations and state changes.
///
/// Note: shared render state (scissor, blend mode, etc.) is carried over when switching between buffers of different types,
/// but shaders and uniforms are not.
#[derive(Default)]
pub struct DrawPool<'a> {
	pool: HashMap<BufferTypeId, Box<dyn IDrawBuffer<'a> + 'a>>,
	subs: Vec<DrawSub>,
}

impl<'a> DrawPool<'a> {
	/// Creates a new command buffer pool.
	#[inline]
	pub fn new() -> DrawPool<'a> {
		DrawPool {
			pool: HashMap::new(),
			subs: Vec::new(),
		}
	}

	/// Clears the command buffer pool for reuse.
	pub fn clear(&mut self) {
		for buf in self.pool.values_mut() {
			buf.clear();
		}
		self.subs.clear();
	}

	/// Returns a compatible [DrawBuilder] for issuing draw commands.
	///
	/// If the most recently used buffer matches the requested vertex and uniform types,
	/// it is reused to continue adding geometry efficiently.
	///
	/// When switching to a different buffer type, shared render state (scissor, blend mode, etc.)
	/// is preserved from the previous buffer to maintain consistency.
	///
	/// Note that shader and shader uniforms are *not* carried over and must be set explicitly.
	pub fn get<V: TVertex, U: TUniform + BufferTypeKey + 'a>(&mut self) -> &mut DrawBuilder<'a, V, U> {
		let mut shared_state = None;

		// Check the last submission buffer
		if let Some(last) = self.subs.last_mut() {
			let buf = self.pool.get_mut(&last.id).unwrap();

			// If the last buffer is of the correct type, return it
			if let Some(buf) = buf.downcast_mut::<V, U>() {
				// Fixed by -Zpolonius...
				// return buf;
				return unsafe { &mut *(buf as *mut DrawBuilder<'a, V, U>) };
			}

			// Otherwise update its range to include the latest commands
			last.range.end = buf.commands_len();

			// Carry-over shared state from the last buffer
			shared_state = Some(buf.shared_state());
		}

		// Create a new command buffer if it doesn't exist
		let id = BufferTypeId {
			vertex: any::TypeId::of::<V>(),
			uniform: U::key(),
		};
		let buf = self.pool.entry(id)
			.or_insert_with(|| {
				let new_buf = DrawBuilder::<'a, V, U>::new();
				Box::new(new_buf) as Box<dyn IDrawBuffer<'a> + 'a>
			});

		let buf = buf.downcast_mut::<V, U>().unwrap();

		// We switched buffers which may already contain commands,
		// Make sure draw commands are not merged with previous ones
		buf.auto_state_tracking = false;

		// Carry-over shared state if available
		if let Some(shared_state) = shared_state {
			buf.scissor = shared_state.scissor;
			buf.blend_mode = shared_state.blend_mode;
			buf.depth_test = shared_state.depth_test;
			buf.cull_mode = shared_state.cull_mode;
		}

		// Create a new submission
		let cmd_len = buf.commands_len();
		let range = cmd_len..cmd_len;
		self.subs.push(DrawSub { id, range });

		return buf;
	}

	/// Draws all commands in submission order.
	pub fn draw(&mut self, g: &mut Graphics) {
		// Update the last submission range
		if let Some(last) = self.subs.last_mut() {
			if let Some(buf) = self.pool.get_mut(&last.id) {
				last.range.end = buf.commands_len();
			}
		}

		// Upload the buffers
		let data: HashMap<BufferTypeId, DrawData> = self.pool.iter().map(|(&id, buf)| {
			(id, buf.data(g))
		}).collect();

		for sub in &self.subs {
			let buf = self.pool.get(&sub.id).unwrap();
			let data = data.get(&sub.id).unwrap();
			buf.draw_range(g, sub.range.clone(), data);
		}

		// Free the buffers
	}

	/// Draws all commands without preserving submission order.
	///
	/// This can be more efficient when the scene uses depth buffering and does not require blending.
	pub fn draw_unordered(&mut self, g: &mut Graphics) {
		for buf in self.pool.values() {
			buf.draw(g);
		}
	}
}
