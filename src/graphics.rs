use super::*;

/// Arguments for [clear](IGraphics::clear).
#[derive(Default)]
pub struct ClearArgs {
	/// Surface to clear.
	pub surface: Surface,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Bounds2<i32>>,
	/// Color to clear with.
	pub color: Option<cvmath::Vec4f>,
	/// Depth to clear with.
	pub depth: Option<f32>,
	/// Stencil to clear with.
	pub stencil: Option<u8>,
}

/// Draw mask.
pub struct DrawMask {
	pub red: bool,
	pub green: bool,
	pub blue: bool,
	pub alpha: bool,
	pub depth: bool,
	pub stencil: u8,
}
impl DrawMask {
	pub const ALL: Self = Self { red: true, green: true, blue: true, alpha: true, depth: true, stencil: u8::MAX };
	pub const COLOR: Self = Self { red: true, green: true, blue: true, alpha: true, depth: false, stencil: 0 };
	pub const DEPTH: Self = Self { red: false, green: false, blue: false, alpha: false, depth: true, stencil: 0 };
	pub const STENCIL: Self = Self { red: false, green: false, blue: false, alpha: false, depth: false, stencil: u8::MAX };
	pub const NONE: Self = Self { red: false, green: false, blue: false, alpha: false, depth: false, stencil: 0 };
}
impl ops::BitOr<DrawMask> for DrawMask {
	type Output = Self;

	#[inline]
	fn bitor(self, rhs: Self) -> Self::Output {
		Self {
			red: self.red || rhs.red,
			green: self.green || rhs.green,
			blue: self.blue || rhs.blue,
			alpha: self.alpha || rhs.alpha,
			depth: self.depth || rhs.depth,
			stencil: self.stencil | rhs.stencil,
		}
	}
}

/// Arguments for drawing a vertex buffer and metadata.
pub struct DrawVertexBuffer {
	/// Vertex buffer.
	pub buffer: VertexBuffer,
	/// Divisor for instanced rendering.
	pub divisor: VertexDivisor,
}

/// Arguments for [draw](IGraphics::draw).
pub struct DrawArgs<'a> {
	/// Surface to draw on.
	pub surface: Surface,
	/// Viewport rectangle.
	pub viewport: cvmath::Bounds2<i32>,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Bounds2<i32>>,
	/// Blend mode.
	pub blend_mode: BlendMode,
	/// Depth test.
	pub depth_test: Option<DepthTest>,
	/// Triangle culling mode.
	pub cull_mode: Option<CullMode>,
	/// Draw mask.
	pub mask: DrawMask,
	/// Primitive type.
	pub prim_type: PrimType,
	/// Shader used.
	pub shader: Shader,
	/// Uniforms.
	pub uniforms: &'a [&'a dyn UniformVisitor],
	/// Vertex buffers.
	pub vertices: &'a [DrawVertexBuffer],
	/// Index of the first vertex.
	pub vertex_start: u32,
	/// Index of one past the last vertex.
	pub vertex_end: u32,
	/// Number of instances to draw.
	///
	/// If this is less than zero, instanced drawing is disabled.
	pub instances: i32,
}

/// Arguments for [draw_indexed](IGraphics::draw_indexed).
pub struct DrawIndexedArgs<'a> {
	/// Surface to draw on.
	pub surface: Surface,
	/// Viewport rectangle.
	pub viewport: cvmath::Bounds2<i32>,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Bounds2<i32>>,
	/// Blend mode.
	pub blend_mode: BlendMode,
	/// Depth test.
	pub depth_test: Option<DepthTest>,
	/// Triangle culling mode.
	pub cull_mode: Option<CullMode>,
	/// Draw mask.
	pub mask: DrawMask,
	/// Primitive type.
	pub prim_type: PrimType,
	/// Shader used.
	pub shader: Shader,
	/// Uniforms.
	pub uniforms: &'a [&'a dyn UniformVisitor],
	/// Vertices.
	pub vertices: &'a [DrawVertexBuffer],
	/// Indices.
	pub indices: IndexBuffer,
	/// Index of the first index.
	pub index_start: u32,
	/// Index of one past the last index.
	pub index_end: u32,
	/// Number of instances to draw.
	///
	/// If this is less than zero, instanced drawing is disabled.
	pub instances: i32,
}

/// Graphics error.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GfxError {
	InvalidHandle,
	IndexOutOfBounds,
	InvalidDrawCallTime,
	ShaderCompileError,
	NameNotFound,
	InternalError,
}

/// Free mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FreeMode {
	/// Delete the resource immediately.
	Delete,
	/// Release the resource, but keep the handle valid for future use.
	Release,
}

/// Graphics interface.
///
/// See [`Graphics`](struct.Graphics.html) for a type-erased version.
pub trait IGraphics {
	/// Begin drawing.
	fn begin(&mut self) -> Result<(), GfxError>;
	/// Clear the surface.
	fn clear(&mut self, args: &ClearArgs) -> Result<(), GfxError>;
	/// Draw primitives.
	fn draw(&mut self, args: &DrawArgs) -> Result<(), GfxError>;
	/// Draw indexed primitives.
	fn draw_indexed(&mut self, args: &DrawIndexedArgs) -> Result<(), GfxError>;
	/// End drawing.
	fn end(&mut self) -> Result<(), GfxError>;

	/// Create a buffer.
	fn vertex_buffer_create(&mut self, name: Option<&str>, size: usize, layout: &'static VertexLayout, usage: BufferUsage) -> Result<VertexBuffer, GfxError>;
	/// Find a vertex buffer by name.
	fn vertex_buffer_find(&mut self, name: &str) -> Result<VertexBuffer, GfxError>;
	/// Set the data of a vertex buffer.
	fn vertex_buffer_set_data(&mut self, id: VertexBuffer, data: &[u8]) -> Result<(), GfxError>;
	/// Release the resources of a vertex buffer.
	fn vertex_buffer_free(&mut self, id: VertexBuffer, mode: FreeMode) -> Result<(), GfxError>;

	/// Create a buffer.
	fn index_buffer_create(&mut self, name: Option<&str>, size: usize, index_ty: IndexType, usage: BufferUsage) -> Result<IndexBuffer, GfxError>;
	/// Find a vertex buffer by name.
	fn index_buffer_find(&mut self, name: &str) -> Result<IndexBuffer, GfxError>;
	/// Set the data of a vertex buffer.
	fn index_buffer_set_data(&mut self, id: IndexBuffer, data: &[u8]) -> Result<(), GfxError>;
	/// Release the resources of a vertex buffer.
	fn index_buffer_free(&mut self, id: IndexBuffer, mode: FreeMode) -> Result<(), GfxError>;

	/// Create and compile a shader.
	fn shader_create(&mut self, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Result<Shader, GfxError>;
	/// Find a shader by name.
	fn shader_find(&mut self, name: &str) -> Result<Shader, GfxError>;
	/// Release the resources of a shader.
	fn shader_free(&mut self, id: Shader) -> Result<(), GfxError>;

	/// Create a 2D texture.
	fn texture2d_create(&mut self, name: Option<&str>, info: &Texture2DInfo) -> Result<Texture2D, GfxError>;
	/// Find a 2D texture by name.
	fn texture2d_find(&mut self, name: &str) -> Result<Texture2D, GfxError>;
	/// Set the data of a 2D texture.
	fn texture2d_set_data(&mut self, id: Texture2D, data: &[u8]) -> Result<(), GfxError>;
	/// Get the info of a 2D texture.
	fn texture2d_get_info(&mut self, id: Texture2D) -> Result<Texture2DInfo, GfxError>;
	/// Release the resources of a 2D texture.
	fn texture2d_free(&mut self, id: Texture2D, mode: FreeMode) -> Result<(), GfxError>;

	/// Create a surface.
	fn surface_create(&mut self, name: Option<&str>, info: &SurfaceInfo) -> Result<Surface, GfxError>;
	/// Find a surface by name.
	fn surface_find(&mut self, name: &str) -> Result<Surface, GfxError>;
	/// Get the info of a surface.
	fn surface_get_info(&mut self, id: Surface) -> Result<SurfaceInfo, GfxError>;
	/// Set the info of a surface.
	fn surface_set_info(&mut self, id: Surface, info: &SurfaceInfo) -> Result<(), GfxError>;
	/// Get the texture of a surface.
	fn surface_get_texture(&mut self, id: Surface) -> Result<Texture2D, GfxError>;
	/// Release the resources of a surface.
	fn surface_free(&mut self, id: Surface, mode: FreeMode) -> Result<(), GfxError>;
}

/// Graphics interface.
///
/// Adds helper methods to the [IGraphics](IGraphics) interface.
#[repr(transparent)]
pub struct Graphics {
	inner: dyn IGraphics,
}

/// Graphics constructor.
#[allow(non_snake_case)]
#[inline]
pub fn Graphics(g: &mut dyn IGraphics) -> &mut Graphics {
	unsafe { mem::transmute(g) }
}

impl ops::Deref for Graphics {
	type Target = dyn IGraphics;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
impl ops::DerefMut for Graphics {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl Graphics {
	/// Create and assign data to a vertex buffer.
	#[inline]
	pub fn vertex_buffer<T: TVertex>(&mut self, name: Option<&str>, data: &[T], usage: BufferUsage) -> Result<VertexBuffer, GfxError> {
		let this = &mut **self;
		let id = this.vertex_buffer_create(name, mem::size_of_val(data), T::LAYOUT, usage)?;
		if let Err(err) = this.vertex_buffer_set_data(id, dataview::bytes(data)) {
			// If setting data fails, delete the buffer and return the error.
			let _ = this.vertex_buffer_free(id, FreeMode::Delete);
			return Err(err);
		}
		Ok(id)
	}
	/// Set the data of a vertex buffer.
	#[inline]
	pub fn buffer_set_data<T: TVertex>(&mut self, id: VertexBuffer, data: &[T]) -> Result<(), GfxError> {
		self.inner.vertex_buffer_set_data(id, dataview::bytes(data))
	}
	/// Create and assign data to an index buffer.
	#[inline]
	pub fn index_buffer<T: TIndex>(&mut self, name: Option<&str>, data: &[T], nverts: T, usage: BufferUsage) -> Result<IndexBuffer, GfxError> {
		#[cfg(debug_assertions)]
		if nverts != Default::default() {
			for i in 0..data.len() {
				if data[i] >= nverts {
					return Err(GfxError::IndexOutOfBounds);
				}
			}
		}
		let this = &mut **self;
		let id = this.index_buffer_create(name, mem::size_of_val(data), T::TYPE, usage)?;
		if let Err(err) = this.index_buffer_set_data(id, dataview::bytes(data)) {
			// If setting data fails, delete the buffer and return the error.
			let _ = this.index_buffer_free(id, FreeMode::Delete);
			return Err(err);
		}
		Ok(id)
	}
	/// Set the data of an index buffer.
	#[inline]
	pub fn index_buffer_set_data<T: TIndex>(&mut self, id: IndexBuffer, data: &[T]) -> Result<(), GfxError> {
		self.inner.index_buffer_set_data(id, dataview::bytes(data))
	}
}
