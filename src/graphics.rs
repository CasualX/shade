use super::*;

/// Arguments for [begin](IGraphics::begin).
pub enum BeginArgs<'a> {
	/// Begin drawing on the back buffer.
	BackBuffer {
		viewport: cvmath::Bounds2<i32>,
	},
	/// Begin immediate mode drawing.
	Immediate {
		color: &'a [Texture2D],
		depth: Texture2D,
		viewport: cvmath::Bounds2<i32>,
	},
}

/// Arguments for [clear](IGraphics::clear).
#[derive(Default)]
pub struct ClearArgs {
	/// Scissor rectangle.
	pub scissor: Option<cvmath::Bounds2<i32>>,
	/// Color to clear with.
	pub color: Option<cvmath::Vec4f>,
	/// Depth to clear with.
	pub depth: Option<f32>,
	/// Stencil to clear with.
	pub stencil: Option<u8>,
}

/// Clear the surface.
#[macro_export]
macro_rules! clear {
	($g:expr $(, $field:ident : $value:expr )* $(,)? ) => {
		$g.clear(&$crate::ClearArgs {
			$( $field: Some($value), )*
			..Default::default()
		});
	};
}

/// Draw mask.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

/// Free mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FreeMode {
	/// Delete the resource immediately.
	Delete,
	/// Release the resource, but keep the handle valid for future use.
	Release,
}

/// Drawing statistics.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct DrawMetrics {
	/// Time spent between `begin()` and `end()`.
	pub draw_duration: time::Duration,
	/// Number of `draw()` and `draw_indexed()` calls.
	pub draw_call_count: u32,
	/// Number of vertices drawn.
	pub vertex_count: u32,
	/// Number of bytes uploaded to the GPU.
	pub bytes_uploaded: usize,
	/// Number of bytes downloaded from the GPU.
	pub bytes_downloaded: usize,
}

/// Graphics interface.
///
/// See [Graphics](struct@Graphics) for a type-erased version.
pub trait IGraphics {
	/// Begin drawing.
	fn begin(&mut self, args: &BeginArgs);
	/// Clear the surface.
	fn clear(&mut self, args: &ClearArgs);
	/// Draw primitives.
	fn draw(&mut self, args: &DrawArgs);
	/// Draw indexed primitives.
	fn draw_indexed(&mut self, args: &DrawIndexedArgs);
	/// End drawing.
	fn end(&mut self);
	/// Get drawing statistics.
	fn get_draw_metrics(&mut self, reset: bool) -> DrawMetrics;

	/// Creates a vertex buffer.
	fn vertex_buffer_create(&mut self, name: Option<&str>, size: usize, layout: &'static VertexLayout, usage: BufferUsage) -> VertexBuffer;
	/// Finds a vertex buffer by name.
	fn vertex_buffer_find(&self, name: &str) -> VertexBuffer;
	/// Writes data to the vertex buffer.
	fn vertex_buffer_write(&mut self, id: VertexBuffer, data: &[u8]);
	/// Releases the resources of the vertex buffer.
	fn vertex_buffer_free(&mut self, id: VertexBuffer, mode: FreeMode);

	/// Creates an index buffer.
	fn index_buffer_create(&mut self, name: Option<&str>, size: usize, index_ty: IndexType, usage: BufferUsage) -> IndexBuffer;
	/// Finds an index buffer by name.
	fn index_buffer_find(&self, name: &str) -> IndexBuffer;
	/// Writes data to the index buffer.
	fn index_buffer_write(&mut self, id: IndexBuffer, data: &[u8]);
	/// Releases the resources of the index buffer.
	fn index_buffer_free(&mut self, id: IndexBuffer, mode: FreeMode);

	/// Creates and compiles a shader.
	fn shader_create(&mut self, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Shader;
	/// Finds a shader by name.
	fn shader_find(&self, name: &str) -> Shader;
	/// Releases the resources of the shader.
	fn shader_free(&mut self, id: Shader);

	/// Creates a 2D texture.
	fn texture2d_create(&mut self, name: Option<&str>, info: &Texture2DInfo) -> Texture2D;
	/// Finds a 2D texture by name.
	fn texture2d_find(&self, name: &str) -> Texture2D;
	/// Returns the info of the 2D texture.
	fn texture2d_get_info(&self, id: Texture2D) -> Option<&Texture2DInfo>;
	/// Generates mipmap for the 2D texture.
	fn texture2d_generate_mipmap(&mut self, id: Texture2D);
	/// Updates the properties of the 2D texture and reallocates the texture if necessary.
	fn texture2d_update(&mut self, id: Texture2D, info: &Texture2DInfo) -> Texture2D;
	/// Writes data to the 2D texture.
	fn texture2d_write(&mut self, id: Texture2D, level: u8, data: &[u8]);
	/// Reads the data of the 2D texture into the buffer.
	fn texture2d_read_into(&mut self, id: Texture2D, level: u8, data: &mut [u8]);
	/// Releases the resources of the 2D texture.
	fn texture2d_free(&mut self, id: Texture2D, mode: FreeMode);
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
	/// Creates a texture from an image.
	#[inline]
	pub fn image<F: ImageToTexture>(&mut self, name: Option<&str>, image: &F) -> Texture2D {
		let info = image.info();
		let data = image.data();
		let tex = self.texture2d_create(name, &info);
		self.texture2d_write(tex, 0, data);
		self.texture2d_generate_mipmap(tex);
		return tex;
	}
	/// Creates an animated texture from an animated image.
	pub fn animated_image(&mut self, image: &image::AnimatedImage, props: &TextureProps) -> AnimatedTexture2D {
		let mut frames = Vec::with_capacity(image.frames.len());
		for frame in &image.frames {
			let tex = {
				let info = Texture2DInfo {
					format: TextureFormat::SRGBA8,
					width: frame.width,
					height: frame.height,
					props: *props,
				};
				let tex = self.texture2d_create(None, &info);
				self.texture2d_write(tex, 0, frame.as_bytes());
				self.texture2d_generate_mipmap(tex);
				tex
			};
			frames.push(tex);
		}
		let length = image.delays.iter().sum();
		AnimatedTexture2D {
			width: image.width,
			height: image.height,
			frames,
			length,
			repeat: image.repeat,
		}
	}
	/// Creates and writes data to the 2D texture.
	#[inline]
	pub fn texture2d(&mut self, name: Option<&str>, info: &Texture2DInfo, data: &[u8]) -> Texture2D {
		let texture = self.texture2d_create(name, info);
		self.texture2d_write(texture, 0, data);
		self.texture2d_generate_mipmap(texture);
		return texture;
	}
	/// Reads the data of the 2D texture into an image.
	pub fn texture2d_read<T: dataview::Pod>(&mut self, id: Texture2D, level: u8) -> image::Image<T> {
		let info = self.texture2d_get_info(id).unwrap();
		let (mip_width, mip_height, byte_size) = info.mip_size(level);
		assert!(byte_size % mem::size_of::<T>() == 0, "Texture2D level={level} byte_size={byte_size} is not a multiple of {}", mem::size_of::<T>());
		let nelements = byte_size / mem::size_of::<T>();
		let mut vec = Vec::with_capacity(nelements);
		let data = unsafe { slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, byte_size) };
		self.texture2d_read_into(id, level, data);
		unsafe { vec.set_len(nelements) };
		return image::Image {
			width: mip_width,
			height: mip_height,
			data: vec,
		};
	}
	/// Creates and writes data to the vertex buffer.
	#[inline]
	pub fn vertex_buffer<T: TVertex>(&mut self, name: Option<&str>, data: &[T], usage: BufferUsage) -> VertexBuffer {
		let this = &mut self.inner;
		let id = this.vertex_buffer_create(name, mem::size_of_val(data), T::LAYOUT, usage);
		this.vertex_buffer_write(id, dataview::bytes(data));
		return id;
	}
	/// Writes data to the vertex buffer.
	#[inline]
	pub fn vertex_buffer_write<T: TVertex>(&mut self, id: VertexBuffer, data: &[T]) {
		self.inner.vertex_buffer_write(id, dataview::bytes(data))
	}
	/// Creates and writes data to the index buffer.
	#[inline]
	pub fn index_buffer<T: TIndex>(&mut self, name: Option<&str>, data: &[T], _nverts: T, usage: BufferUsage) -> IndexBuffer {
		#[cfg(debug_assertions)]
		if _nverts != Default::default() {
			for i in 0..data.len() {
				if data[i] >= _nverts {
					panic!("Index {:?} out of bounds for {:?} vertices", data[i], _nverts);
				}
			}
		}
		let this = &mut self.inner;
		let id = this.index_buffer_create(name, mem::size_of_val(data), T::TYPE, usage);
		this.index_buffer_write(id, dataview::bytes(data));
		return id;
	}
	/// Writes data to the index buffer.
	#[inline]
	pub fn index_buffer_write<T: TIndex>(&mut self, id: IndexBuffer, data: &[T]) {
		self.inner.index_buffer_write(id, dataview::bytes(data))
	}
}
