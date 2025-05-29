use super::*;

/// Arguments for [clear](IGraphics::clear).
#[derive(Default)]
pub struct ClearArgs {
	/// Surface to clear.
	pub surface: Surface,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Rect<i32>>,
	/// Color to clear with.
	pub color: Option<cvmath::Vec4<f32>>,
	/// Depth to clear with.
	pub depth: Option<f32>,
	/// Stencil to clear with.
	pub stencil: Option<u32>,
}

/// Arguments for drawing a vertex buffer and metadata.
pub struct DrawVertexBuffer {
	/// Vertex buffer.
	pub buffer: Buffer,
	/// Divisor for instanced rendering.
	pub divisor: VertexDivisor,
	/// Reference to the vertex layout that describes the structure of the vertex data.
	pub layout: &'static VertexLayout,
}

/// Arguments for [draw](IGraphics::draw).
pub struct DrawArgs<'a> {
	/// Surface to draw on.
	pub surface: Surface,
	/// Viewport rectangle.
	pub viewport: cvmath::Rect<i32>,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Rect<i32>>,
	/// Blend mode.
	pub blend_mode: BlendMode,
	/// Depth test.
	pub depth_test: Option<DepthTest>,
	/// Triangle culling mode.
	pub cull_mode: Option<CullMode>,
	/// Primitive type.
	pub prim_type: PrimType,
	/// Shader used.
	pub shader: Shader,
	/// Vertex buffers.
	pub vertices: &'a [DrawVertexBuffer],
	/// Uniforms.
	pub uniforms: UniformRef<'a>,
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
	pub viewport: cvmath::Rect<i32>,
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Rect<i32>>,
	/// Blend mode.
	pub blend_mode: BlendMode,
	/// Depth test.
	pub depth_test: Option<DepthTest>,
	/// Triangle culling mode.
	pub cull_mode: Option<CullMode>,
	/// Primitive type.
	pub prim_type: PrimType,
	/// Shader used.
	pub shader: Shader,
	/// Vertices.
	pub vertices: &'a [DrawVertexBuffer],
	/// Indices.
	pub indices: Buffer,
	/// Index type.
	pub index_type: IndexType,
	/// Uniforms.
	pub uniforms: UniformRef<'a>,
	/// Index of the first vertex.
	pub vertex_start: u32,
	/// Index of one past the last vertex.
	pub vertex_end: u32,
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
	fn buffer_create(&mut self, name: Option<&str>, size: usize, usage: BufferUsage) -> Result<Buffer, GfxError>;
	/// Find a vertex buffer by name.
	fn buffer_find(&mut self, name: &str) -> Result<Buffer, GfxError>;
	/// Set the data of a vertex buffer.
	fn buffer_set_data(&mut self, id: Buffer, data: &[u8]) -> Result<(), GfxError>;
	/// Release the resources of a vertex buffer.
	fn buffer_delete(&mut self, id: Buffer, free_handle: bool) -> Result<(), GfxError>;

	/// Create a shader.
	fn shader_create(&mut self, name: Option<&str>) -> Result<Shader, GfxError>;
	/// Find a shader by name.
	fn shader_find(&mut self, name: &str) -> Result<Shader, GfxError>;
	/// Compile a shader.
	fn shader_compile(&mut self, id: Shader, vertex_source: &str, fragment_source: &str) -> Result<(), GfxError>;
	/// Get the compile log of a shader.
	fn shader_compile_log(&mut self, id: Shader) -> Result<String, GfxError>;
	/// Release the resources of a shader.
	fn shader_delete(&mut self, id: Shader, free_handle: bool) -> Result<(), GfxError>;

	/// Create a 2D texture.
	fn texture2d_create(&mut self, name: Option<&str>, info: &Texture2DInfo) -> Result<Texture2D, GfxError>;
	/// Find a 2D texture by name.
	fn texture2d_find(&mut self, name: &str) -> Result<Texture2D, GfxError>;
	/// Set the data of a 2D texture.
	fn texture2d_set_data(&mut self, id: Texture2D, data: &[u8]) -> Result<(), GfxError>;
	/// Get the info of a 2D texture.
	fn texture2d_get_info(&mut self, id: Texture2D) -> Result<Texture2DInfo, GfxError>;
	/// Release the resources of a 2D texture.
	fn texture2d_delete(&mut self, id: Texture2D, free_handle: bool) -> Result<(), GfxError>;

	/// Create a 2D texture array.
	fn texture2darray_create(&mut self, name: Option<&str>, info: &Texture2DArrayInfo) -> Result<Texture2DArray, GfxError>;
	/// Find a 2D texture array by name.
	fn texture2darray_find(&mut self, name: &str) -> Result<Texture2DArray, GfxError>;
	/// Set the data of a 2D texture array.
	fn texture2darray_set_data(&mut self, id: Texture2DArray, index: usize, data: &[u8]) -> Result<(), GfxError>;
	/// Get the info of a 2D texture array.
	fn texture2darray_get_info(&mut self, id: Texture2DArray) -> Result<Texture2DArrayInfo, GfxError>;
	/// Get the depth of a 2D texture array.
	fn texture2darray_delete(&mut self, id: Texture2DArray, free_handle: bool) -> Result<(), GfxError>;

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
	fn surface_delete(&mut self, id: Surface, free_handle: bool) -> Result<(), GfxError>;
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
	/// Create and assign data to a buffer.
	#[inline]
	pub fn buffer<T: dataview::Pod>(&mut self, name: Option<&str>, data: &[T], usage: BufferUsage) -> Result<Buffer, GfxError> {
		let this = &mut **self;
		let id = this.buffer_create(name, mem::size_of_val(data), usage)?;
		if let Err(err) = this.buffer_set_data(id, dataview::bytes(data)) {
			// If setting data fails, delete the buffer and return the error.
			let _ = this.buffer_delete(id, true);
			return Err(err);
		}
		Ok(id)
	}
	/// Set the data of a buffer.
	#[inline]
	pub fn buffer_set_data<T: dataview::Pod>(&mut self, id: Buffer, data: &[T]) -> Result<(), GfxError> {
		self.inner.buffer_set_data(id, dataview::bytes(data))
	}
}
