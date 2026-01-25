/*!
OpenGL graphics backend.
*/

use std::{mem, ops, ptr, slice, time};
use std::any::type_name_of_val as name_of;
use std::collections::{hash_map, HashMap};

/// Re-exported OpenGL bindings.
pub use gl as capi;
use gl::types::*;

type NameBuf = crate::sstring::SmallString<64>;

#[cfg(debug_assertions)]
macro_rules! gl_check {
	(gl::$f:ident($($e:expr),*)) => {
		check(|| {
			// println!(concat!("gl", stringify!($f), "(", $(stringify!($e),"={:?}, ",)* ")"), $($e),*);
			unsafe { gl::$f($($e),*) }
		})
	};
}
#[cfg(not(debug_assertions))]
macro_rules! gl_check {
	($e:expr) => {
		unsafe { $e }
	};
}

mod cvt;
mod draw;
mod shader;
mod texture2d;
mod objects;

use self::cvt::*;
use self::objects::*;

pub mod shaders;

#[derive(Clone, Debug)]
pub struct GlConfig {
	pub srgb: bool,
}
impl Default for GlConfig {
	fn default() -> Self {
		GlConfig {
			srgb: true,
		}
	}
}

pub struct GlGraphics {
	objects: ObjectMap,
	texture2d_default: crate::Texture2D,
	dynamic_vao: GLuint,
	drawing: bool,
	draw_begin: time::Instant,
	metrics: crate::DrawMetrics,
	/// Temporary framebuffer for immediate render passes, deleted at end().
	immediate_fbo: Option<GLuint>,
	config: GlConfig,
}

impl GlGraphics {
	pub fn new(config: GlConfig) -> Self {
		let mut dynamic_vao = 0;
		gl_check!(gl::GenVertexArrays(1, &mut dynamic_vao));

		if config.srgb {
			gl_check!(gl::Enable(gl::FRAMEBUFFER_SRGB));
		}
		else {
			gl_check!(gl::Disable(gl::FRAMEBUFFER_SRGB));
		}

		let mut objects = ObjectMap::new();

		let mut default_texture2d = 0;
		{
			gl_check!(gl::GenTextures(1, &mut default_texture2d));
			gl_check!(gl::BindTexture(gl::TEXTURE_2D, default_texture2d));
			let color = [0u8; 4];
			gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, color.as_ptr() as *const GLvoid));
			gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		}
		let texture2d_default = GlTexture2D {
			texture: default_texture2d,
			info: crate::Texture2DInfo {
				format: crate::TextureFormat::RGBA8,
				width: 1,
				height: 1,
				props: crate::TextureProps {
					mip_levels: 1,
					usage: crate::TextureUsage::TEXTURE,
					filter_min: crate::TextureFilter::Nearest,
					filter_mag: crate::TextureFilter::Nearest,
					wrap_u: crate::TextureWrap::Edge,
					wrap_v: crate::TextureWrap::Edge,
					compare: None,
					border_color: [0.0, 0.0, 0.0, 0.0],
				}
			},
		};
		let default_texture2d_handle = objects.insert(texture2d_default);


		GlGraphics {
			objects,
			texture2d_default: default_texture2d_handle,
			dynamic_vao,
			drawing: false,
			draw_begin: time::Instant::now(),
			metrics: Default::default(),
			immediate_fbo: None,
			config,
		}
	}
	/// Returns the graphics interface.
	#[inline]
	pub fn as_graphics(&mut self) -> &mut crate::Graphics {
		crate::Graphics(self)
	}
}

impl crate::IGraphics for GlGraphics {
	fn get_type(&self, object: crate::BaseObject) -> Option<crate::ObjectType> {
		self.objects.get_type(object)
	}
	fn add_ref(&mut self, object: crate::BaseObject) {
		self.objects.add_ref(object);
	}
	fn release(&mut self, object: crate::BaseObject) -> u32 {
		self.objects.release(object)
	}

	fn begin(&mut self, args: &crate::BeginArgs) {
		draw::begin(self, args)
	}

	fn clear(&mut self, args: &crate::ClearArgs) {
		draw::clear(self, args)
	}

	fn draw(&mut self, args: &crate::DrawArgs) {
		draw::arrays(self, args)
	}

	fn draw_indexed(&mut self, args: &crate::DrawIndexedArgs) {
		draw::indexed(self, args)
	}

	fn end(&mut self) {
		draw::end(self)
	}

	fn get_draw_metrics(&mut self, reset: bool) -> crate::DrawMetrics {
		if reset {
			mem::take(&mut self.metrics)
		}
		else {
			self.metrics
		}
	}

	fn vertex_buffer_create(&mut self, size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> crate::VertexBuffer {
		// Ensure non-zero size? What to do with zero sized buffers?
		let alloc_size = if size == 0 { 1 } else { size };

		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buffer));
		let gl_usage = match usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BufferData(gl::ARRAY_BUFFER, alloc_size as GLsizeiptr, ptr::null(), gl_usage));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));

		self.objects.insert(GlVertexBuffer { buffer, size, _usage: usage, layout })
	}

	fn vertex_buffer_write(&mut self, id: crate::VertexBuffer, offset: usize, data: &[u8]) {
		let Some(buf) = self.objects.get_vertex_buffer(id) else { return };
		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Vertex buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferSubData(gl::ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr() as *const _));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
	}

	fn index_buffer_create(&mut self, size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> crate::IndexBuffer {
		// Ensure non-zero size? What to do with zero sized buffers?
		let alloc_size = if size == 0 { 1 } else { size };

		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer));
		let gl_usage = match usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, alloc_size as GLsizeiptr, ptr::null(), gl_usage));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

		self.objects.insert(GlIndexBuffer { buffer, size, _usage: usage, ty })
	}

	fn index_buffer_write(&mut self, id: crate::IndexBuffer, offset: usize, data: &[u8]) {
		let Some(buf) = self.objects.get_index_buffer(id) else { return };
		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Index buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr() as *const _));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
	}

	fn shader_compile(&mut self, vertex_source: &str, fragment_source: &str) -> crate::ShaderProgram {
		shader::compile(self, vertex_source, fragment_source)
	}

	fn texture2d_create(&mut self, info: &crate::Texture2DInfo) -> crate::Texture2D {
		texture2d::create(self, info)
	}

	fn texture2d_get_info(&self, id: crate::Texture2D) -> Option<&crate::Texture2DInfo> {
		texture2d::get_info(self, id)
	}

	fn texture2d_generate_mipmap(&mut self, id: crate::Texture2D) {
		texture2d::generate_mipmap(self, id)
	}

	fn texture2d_update(&mut self, id: crate::Texture2D, info: &crate::Texture2DInfo) -> crate::Texture2D {
		texture2d::update(self, id, info)
	}

	fn texture2d_write(&mut self, id: crate::Texture2D, level: u8, data: &[u8]) {
		texture2d::write(self, id, level, data)
	}

	fn texture2d_read_into(&mut self, id: crate::Texture2D, level: u8, data: &mut [u8]) {
		texture2d::read_into(self, id, level, data)
	}
}

impl ops::Deref for GlGraphics {
	type Target = crate::Graphics;

	#[inline]
	fn deref(&self) -> &crate::Graphics {
		unsafe { mem::transmute(self as &dyn crate::IGraphics) }
	}
}
impl ops::DerefMut for GlGraphics {
	#[inline]
	fn deref_mut(&mut self) -> &mut crate::Graphics {
		crate::Graphics(self)
	}
}

#[cfg(debug_assertions)]
#[inline]
#[track_caller]
fn check<T, F: FnOnce() -> T>(f: F) -> T {
	let result = f();

	let mut nerrors = 0;
	loop {
		let error = unsafe { gl::GetError() };
		if error == gl::NO_ERROR {
			break;
		}
		nerrors += 1;
		eprintln!("OpenGL error: {:#X}", error);
	}

	if nerrors > 0 {
		panic!("OpenGL check failed with {} error(s)", nerrors);
	}

	result
}
