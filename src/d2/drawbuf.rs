use super::*;

type IndexT = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineState {
	prim_type: PrimType,
	viewport: Bounds2<i32>,
	scissor: Option<Bounds2<i32>>,
	blend_mode: BlendMode,
	depth_test: Option<DepthTest>,
	cull_mode: Option<CullMode>,
	mask: DrawMask,
	shader: Shader,
	uniform_index: u32,
}

pub struct Command {
	pipeline_state: PipelineState,
	index_start: u32,
	index_end: u32,
}

/// Draw buffer.
///
/// A `DrawBuffer` collects geometry (vertices and indices) generated on the CPU, and prepares
/// it for rendering. It is the primary structure for immediate-mode drawing.
///
/// See also: [`DrawPool`] allows mixing multiple `DrawBuffer` types with different vertex or uniform formats in a single pass.
///
/// ### Filling the buffer
///
/// Use one of the high-level tools to populate the buffer with geometry:
///
/// - [`Paint`] – fill shapes with color via `fill_*` methods.
/// - [`Pen`] – draw outlines and polylines via `draw_*` methods.
/// - [`Sprite`] – draw textured quads and images via `sprite_*` methods.
/// - [`Scribe`] – draw text via `text_*` methods.
///
/// These tools append to the buffer incrementally and can be freely mixed.
///
/// ### Configuring rendering
///
/// Rendering behavior is controlled via the public fields on this type:
///
/// - `viewport` **must** be set before drawing.
/// - `blend_mode`, `depth_test`, `scissor`, `cull_mode`, and `shader` can be customized.
/// - The `uniform` field contains shader-specific uniform data.
///
/// Draw calls are automatically batched when these settings remain unchanged. For best
/// performance, group similar drawing operations together using the same tool and properties.
///
/// ### Advanced usage
///
/// #### Reusing the buffer
///
/// A `DrawBuffer` can be reused across frames to avoid repeated allocations. Simply call
/// [`clear()`](Self::clear) to remove previously submitted geometry and start fresh.
///
/// #### Drawing multiple times
///
/// Once populated, a `DrawBuffer` can be drawn multiple times using [`draw()`](Self::draw).
/// This is useful when the same content needs to be rendered to different surfaces or at
/// different points in the frame. The buffer remains valid until cleared or modified.
///
/// #### Cross-thread generation
///
/// Geometry can be generated in a background thread by filling a `DrawBuffer`, then sending
/// it to the main thread for rendering. This allows expensive CPU-side work (e.g., shape
/// tessellation, layout) to be decoupled from real-time rendering.
///
/// #### Custom draw tools
///
/// For low-level or specialized use cases, custom draw tools can be implemented using the
/// [`begin()`](Self::begin) method. This provides access to a [`PrimBuilder`], allowing direct
/// construction of geometry and command batching.
///
/// ### Example
///
/// ```
/// use shade::{cvmath, d2};
///
/// fn draw(g: &mut shade::Graphics, viewport: cvmath::Bounds2i, shader: shade::Shader) {
/// 	// Construct a new draw buffer instance
/// 	let mut cv = d2::DrawBuffer::<d2::ColorVertex, d2::ColorUniform>::new();
///
/// 	// Adjust the shared properties
/// 	cv.viewport = viewport;
/// 	cv.blend_mode = shade::BlendMode::Alpha;
/// 	cv.shader = shader;
///
/// 	// Setup shader uniforms
/// 	cv.uniform.transform = cvmath::Transform2::ortho(viewport.cast());
///
/// 	// Create a paint bucket with the vertex template
/// 	let paint = d2::Paint {
/// 		template: d2::ColorTemplate {
/// 			color1: cvmath::Vec4(255, 128, 128, 190),
/// 			color2: cvmath::Vec4::dup(255),
/// 		},
/// 	};
///
/// 	// Using the paint bucket, fill some shapes
/// 	cv.fill_edge_rect(&paint, &cvmath::Bounds2::c(50.0, 50.0, 100.0, 100.0), 0.2);
/// 	cv.fill_ring(&paint, &cvmath::Bounds2::c(50.0, 50.0, 150.0, 150.0), 5.0, 19);
///
/// 	// Draw to the back buffer
/// 	cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
/// }
/// ```
pub struct DrawBuffer<V, U> {
	pub(super) vertices: Vec<V>,
	pub(super) indices: Vec<IndexT>,
	pub(super) uniforms: Vec<U>,
	pub(super) commands: Vec<Command>,
	pub(super) auto_state_tracking: bool,

	pub viewport: Bounds2<i32>,
	pub scissor: Option<Bounds2<i32>>,
	pub blend_mode: BlendMode,
	pub depth_test: Option<DepthTest>,
	pub cull_mode: Option<CullMode>,

	pub shader: Shader,
	pub uniform: U,
}

pub struct DrawBatch<'a> {
	pub commands: &'a [Command],
	pub vertices: VertexBuffer,
	pub indices: IndexBuffer,
}

impl<V: TVertex, U: TUniform> DrawBuffer<V, U> {
	/// Creates a new draw buffer.
	#[inline]
	pub fn new() -> Self {
		DrawBuffer {
			vertices: Vec::new(),
			indices: Vec::new(),
			uniforms: Vec::new(),
			commands: Vec::new(),
			auto_state_tracking: false,

			viewport: Bounds2::ZERO,
			scissor: None,
			blend_mode: BlendMode::Solid,
			depth_test: None,
			cull_mode: None,

			shader: Shader::INVALID,
			uniform: Default::default(),
		}
	}

	/// Clears the draw buffer for reuse.
	#[inline]
	pub fn clear(&mut self) {
		self.vertices.clear();
		self.indices.clear();
		self.uniforms.clear();
		self.commands.clear();
		self.auto_state_tracking = false;

		self.viewport = Bounds2::ZERO;
		self.scissor = None;
		self.blend_mode = BlendMode::Solid;
		self.depth_test = None;
		self.cull_mode = None;

		self.shader = Shader::INVALID;
		self.uniform = Default::default();
	}

	/// Draws the draw buffer.
	pub fn draw(&self, g: &mut Graphics, surface: Surface) -> Result<(), GfxError> {
		let vertices = g.vertex_buffer(None, &self.vertices, BufferUsage::Static)?;
		let indices = g.index_buffer(None, &self.indices, self.vertices.len() as IndexT, BufferUsage::Static)?;
		let range = DrawBatch {
			commands: &self.commands,
			vertices,
			indices,
		};

		let result = self.draw_batch(g, surface, &range);
		g.index_buffer_free(indices, FreeMode::Delete);
		g.vertex_buffer_free(vertices, FreeMode::Delete);
		result
	}

	/// Draws the specified commands from the buffer.
	pub(super) fn draw_batch(&self, g: &mut Graphics, surface: Surface, batch: &DrawBatch) -> Result<(), GfxError> {
		for cmd in batch.commands {
			let uniforms = &self.uniforms[cmd.pipeline_state.uniform_index as usize];
			g.draw_indexed(&DrawIndexedArgs {
				surface,
				viewport: cmd.pipeline_state.viewport,
				scissor: cmd.pipeline_state.scissor,
				blend_mode: cmd.pipeline_state.blend_mode,
				depth_test: cmd.pipeline_state.depth_test,
				cull_mode: cmd.pipeline_state.cull_mode,
				mask: cmd.pipeline_state.mask,
				prim_type: cmd.pipeline_state.prim_type,
				shader: cmd.pipeline_state.shader,
				uniforms: &[uniforms],
				vertices: &[DrawVertexBuffer {
					buffer: batch.vertices,
					divisor: VertexDivisor::PerVertex,
				}],
				indices: batch.indices,
				index_start: cmd.index_start,
				index_end: cmd.index_end,
				instances: -1,
			})?;
		}

		Ok(())
	}

	/// Checks if the uniforms have changed since last time.
	///
	/// If there are no uniforms, or the last uniform is different from the current one,
	/// the current uniforms are pushed to the list of uniforms.
	fn update_uniforms_if_changed(&mut self) {
		let mut new = true;
		if let Some(last) = self.uniforms.last() {
			new = last != &self.uniform;
		}
		if new {
			self.uniforms.push(self.uniform.clone());
		}
	}

	/// Begins adding a new geometry to the draw buffer.
	pub fn begin<'a>(&'a mut self, prim_type: PrimType, nverts: usize, nprims: usize) -> PrimBuilder<'a, V> {
		self.update_uniforms_if_changed();

		let nindices = nprims * match prim_type { PrimType::Triangles => 3, PrimType::Lines => 2, };

		let pipeline_state = PipelineState {
			prim_type,
			viewport: self.viewport,
			scissor: self.scissor,
			blend_mode: self.blend_mode,
			depth_test: self.depth_test,
			cull_mode: self.cull_mode,
			mask: DrawMask::COLOR | DrawMask::DEPTH, // Default mask
			shader: self.shader,
			uniform_index: self.uniforms.len() as u32 - 1,
		};

		// Can we merge this command with the last one?
		let mut new_cmd = true;
		if self.auto_state_tracking {
			if let Some(last) = self.commands.last_mut() {
				if last.pipeline_state == pipeline_state {
					last.index_end += nindices as u32;
					new_cmd = false;
				}
			}
		}

		// Auto track state again if it was disabled
		self.auto_state_tracking = true;

		// Otherwise add a new command
		if new_cmd {
			let index_start = self.indices.len() as u32;
			let index_end = index_start + nindices as u32;
			self.commands.push(Command { pipeline_state, index_start, index_end });
		}

		let vertex_start = self.vertices.len();
		let index_start = self.indices.len();

		// Reserve space for new data.
		self.vertices.resize_with(vertex_start + nverts, V::default);
		debug_assert!(self.vertices.len() <= IndexT::MAX as usize, "too many vertices");
		self.indices.resize(index_start + nindices, vertex_start as IndexT);

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
	indices: &'a mut [IndexT],
	#[cfg(debug_assertions)]
	nvertices: usize,
}

impl<'a, V: TVertex> PrimBuilder<'a, V> {
	/// Adds a vertex index to the command buffer.
	#[track_caller]
	pub fn add_index(&mut self, vertex: u32) {
		#[cfg(debug_assertions)] {
			assert!(self.indices.len() >= 1, "too many indices");
			assert!((vertex as usize) < self.nvertices && vertex < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(1);
		head[0] += vertex as IndexT;
		self.indices = tail;
	}

	/// Adds two vertex indices to the command buffer.
	#[track_caller]
	pub fn add_index2(&mut self, vertex1: u32, vertex2: u32) {
		#[cfg(debug_assertions)] {
			assert!(self.indices.len() >= 2, "too many indices");
			assert!((vertex1 as usize) < self.nvertices && vertex1 < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex1, self.nvertices);
			assert!((vertex2 as usize) < self.nvertices && vertex2 < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex2, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(2);
		head[0] += vertex1 as IndexT;
		head[1] += vertex2 as IndexT;
		self.indices = tail;
	}

	/// Adds three vertex indices to the command buffer.
	#[track_caller]
	pub fn add_index3(&mut self, vertex1: u32, vertex2: u32, vertex3: u32) {
		#[cfg(debug_assertions)] {
			assert!(self.indices.len() >= 3, "too many indices");
			assert!((vertex1 as usize) < self.nvertices && vertex1 < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex1, self.nvertices);
			assert!((vertex2 as usize) < self.nvertices && vertex2 < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex2, self.nvertices);
			assert!((vertex3 as usize) < self.nvertices && vertex3 < IndexT::MAX as u32, "vertex index ({}) out of bounds ({} vertices)", vertex3, self.nvertices);
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(3);
		head[0] += vertex1 as IndexT;
		head[1] += vertex2 as IndexT;
		head[2] += vertex3 as IndexT;
		self.indices = tail;
	}

	/// Adds vertex indices to the command buffer.
	#[track_caller]
	pub fn add_indices(&mut self, indices: &[IndexT]) {
		#[cfg(debug_assertions)] {
			assert!(self.indices.len() >= indices.len(), "too many indices");
			for &index in indices {
				assert!((index as usize) < self.nvertices, "vertex index ({}) out of bounds ({} vertices)", index, self.nvertices);
			}
		}

		let (head, tail) = mem::replace(&mut self.indices, &mut []).split_at_mut(indices.len());
		for i in 0..indices.len() {
			head[i] += indices[i] as IndexT;
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
		#[cfg(debug_assertions)] {
			assert!(self.vertices.len() >= 1, "too many vertices");
		}

		let (head, tail) = mem::replace(&mut self.vertices, &mut []).split_at_mut(1);
		head[0] = vertex;
		self.vertices = tail;
	}

	/// Adds vertices to the command buffer.
	#[track_caller]
	pub fn add_vertices(&mut self, vertices: &[V]) {
		#[cfg(debug_assertions)] {
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
