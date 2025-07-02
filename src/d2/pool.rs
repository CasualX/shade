use std::collections::HashMap;
use std::any;

use super::*;

pub struct SharedState {
	pub viewport: Bounds2<i32>,
	pub scissor: Option<Bounds2<i32>>,
	pub blend_mode: BlendMode,
	pub depth_test: Option<DepthTest>,
	pub cull_mode: Option<CullMode>,
}

struct DrawData {
	vertices: VertexBuffer,
	indices: IndexBuffer,
}

trait IDrawBuffer: any::Any {
	fn clear(&mut self);
	fn data(&self, g: &mut Graphics) -> Result<DrawData, GfxError>;
	fn draw(&self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError>;
	fn draw_batch(&self, g: &mut Graphics, surface: Surface, range: ops::Range<usize>, data: &DrawData) -> Result<(), GfxError>;
	fn commands_len(&self) -> usize;
	fn shared_state(&self) -> SharedState;
}

impl dyn IDrawBuffer {
	#[inline]
	fn downcast_mut<V: TVertex, U: TUniform + 'static>(&mut self) -> Option<&mut DrawBuffer<V, U>> {
		(self as &mut dyn any::Any).downcast_mut::<DrawBuffer<V, U>>()
	}
}

impl<T: TVertex, U: TUniform + 'static> IDrawBuffer for DrawBuffer<T, U> {
	fn clear(&mut self) {
		self.clear();
	}
	fn data(&self, g: &mut Graphics) -> Result<DrawData, GfxError> {
		let vertices = g.vertex_buffer(None, &self.vertices, BufferUsage::Static)?;
		let indices = g.index_buffer(None, &self.indices, self.vertices.len() as u16, BufferUsage::Static)?;
		Ok(DrawData { vertices, indices })
	}
	fn draw(&self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError> {
		self.draw(g, surface)
	}
	fn draw_batch(&self, g: &mut Graphics, surface: Surface, range: ops::Range<usize>, data: &DrawData) -> Result<(), GfxError> {
		let batch = drawbuf::DrawBatch {
			commands: &self.commands[range],
			vertices: data.vertices,
			indices: data.indices,
		};
		self.draw_batch(g, surface, &batch)
	}
	fn commands_len(&self) -> usize {
		self.commands.len()
	}
	fn shared_state(&self) -> SharedState {
		SharedState {
			viewport: self.viewport,
			scissor: self.scissor,
			blend_mode: self.blend_mode,
			depth_test: self.depth_test,
			cull_mode: self.cull_mode,
		}
	}
}

struct DrawSub {
	id: any::TypeId,
	range: ops::Range<usize>,
}

/// Draw buffer pool.
///
/// Manages a collection of [`DrawBuffer`] with heterogeneous vertex and uniform types,
/// enabling efficient reuse and batching of draw commands.
///
/// Use [`get`](Self::get) to obtain a compatible `DrawBuffer<T, U>` for adding geometry.
/// Buffers are reused when possible to minimize allocations and state changes.
///
/// Note: shared render state (viewport, scissor, blend mode, etc.) is carried over when switching between buffers of different types,
/// but shaders and uniforms are not.
#[derive(Default)]
pub struct DrawPool {
	pool: HashMap<any::TypeId, Box<dyn IDrawBuffer>>,
	subs: Vec<DrawSub>,
}

impl DrawPool {
	/// Creates a new command buffer pool.
	#[inline]
	pub fn new() -> DrawPool {
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

	/// Returns a compatible [`DrawBuffer`] for issuing draw commands.
	///
	/// If the most recently used buffer matches the requested vertex and uniform types,
	/// it is reused to continue adding geometry efficiently.
	///
	/// When switching to a different buffer type, shared render state (viewport, scissor, blend mode, etc.)
	/// is preserved from the previous buffer to maintain consistency.
	///
	/// Note that shader and shader uniforms are *not* carried over and must be set explicitly.
	pub fn get<V: TVertex, U: TUniform + 'static>(&mut self) -> &mut DrawBuffer<V, U> {
		let mut shared_state = None;

		// Check the last submission buffer
		if let Some(last) = self.subs.last_mut() {
			let buf = self.pool.get_mut(&last.id).unwrap();

			// If the last buffer is of the correct type, return it
			if let Some(buf) = buf.downcast_mut::<V, U>() {
				// Fixed by -Zpolonius...
				// return buf;
				return unsafe { &mut *(buf as *mut DrawBuffer<V, U>) };
			}

			// Otherwise update its range to include the latest commands
			last.range.end = buf.commands_len();

			// Carry-over shared state from the last buffer
			shared_state = Some(buf.shared_state());
		}

		// Create a new command buffer if it doesn't exist
		let id = any::TypeId::of::<DrawBuffer<V, U>>();
		let buf = self.pool.entry(id)
			.or_insert_with(|| {
				let new_buf = DrawBuffer::<V, U>::new();
				Box::new(new_buf) as Box<dyn IDrawBuffer>
			});

		let buf = buf.downcast_mut::<V, U>().unwrap();

		// We switched buffers which may already contain commands,
		// Make sure draw commands are not merged with previous ones
		buf.auto_state_tracking = false;

		// Carry-over shared state if available
		if let Some(shared_state) = shared_state {
			buf.viewport = shared_state.viewport;
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
	pub fn draw(&mut self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError> {
		// Update the last submission range
		if let Some(last) = self.subs.last_mut() {
			if let Some(buf) = self.pool.get_mut(&last.id) {
				last.range.end = buf.commands_len();
			}
		}

		// Upload the buffers
		let data: Result<HashMap<any::TypeId, DrawData>, _> = self.pool.iter().map(|(&id, buf)| {
			buf.data(g).map(|data| (id, data))
		}).collect();
		let data = data.unwrap();

		for sub in &self.subs {
			let buf = self.pool.get(&sub.id).unwrap();
			let data = data.get(&sub.id).unwrap();
			buf.draw_batch(g, surface, sub.range.clone(), data)?;
		}

		// Free the buffers
		for data in data.values() {
			g.index_buffer_free(data.indices, FreeMode::Delete);
			g.vertex_buffer_free(data.vertices, FreeMode::Delete);
		}

		Ok(())
	}

	/// Draws all commands without preserving submission order.
	///
	/// This can be more efficient when the scene uses depth buffering and does not require blending.
	pub fn draw_unordered(&mut self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError> {
		for buf in self.pool.values() {
			buf.draw(g, surface)?;
		}
		Ok(())
	}
}
