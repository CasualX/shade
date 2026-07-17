/*!
OpenGL graphics backend.
*/

use std::{fmt, mem, ops, ptr, slice, time};
use std::any::type_name_of_val as name_of;
use std::collections::HashMap;

/// Re-exported OpenGL bindings.
pub use gl as capi;
use gl::types::*;

type NameBuf = crate::sstring::SmallString<64>;

mod check;

mod cvt;
mod draw;
mod generation;
mod shader;
mod texture2d;
mod objects;

use self::cvt::*;
use self::objects::*;

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
	generation: u32,
	texture2d_default: GLuint,
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
		let generation = generation::next();

		let mut dynamic_vao = 0;
		gl_check!(gl::GenVertexArrays(1, &mut dynamic_vao));

		let mut texture2d_default = 0;
		gl_check!(gl::GenTextures(1, &mut texture2d_default));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture2d_default));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint));
		let color = [0u8; 4];
		gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, color.as_ptr() as *const GLvoid));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));

		if config.srgb {
			gl_check!(gl::Enable(gl::FRAMEBUFFER_SRGB));
		}
		else {
			gl_check!(gl::Disable(gl::FRAMEBUFFER_SRGB));
		}

		GlGraphics {
			generation,
			texture2d_default,
			dynamic_vao,
			drawing: false,
			draw_begin: time::Instant::now(),
			metrics: Default::default(),
			immediate_fbo: None,
			config,
		}
	}
}

impl Drop for GlGraphics {
	fn drop(&mut self) {
		gl_check!(gl::DeleteTextures(1, &self.texture2d_default));
		gl_check!(gl::DeleteVertexArrays(1, &self.dynamic_vao));
		generation::drop(self.generation);
	}
}

impl crate::IGraphics for GlGraphics {
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

	fn vertex_buffer_create(&mut self, size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> Box<dyn crate::VertexBuffer> {
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
		gl_check!(gl::BufferData(gl::ARRAY_BUFFER, alloc_size as GLsizeiptr, ptr::null::<GLvoid>(), gl_usage));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));

		Box::new(GlVertexBuffer { generation: self.generation, buffer, size, _usage: usage, layout })
	}

	fn vertex_buffer_write(&mut self, buffer: &mut dyn crate::VertexBuffer, offset: usize, data: &[u8]) {
		let buf = objects::vertex_buffer_mut(buffer);
		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Vertex buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferSubData(gl::ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr() as *const _));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
	}

	fn index_buffer_create(&mut self, size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> Box<dyn crate::IndexBuffer> {
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
		gl_check!(gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, alloc_size as GLsizeiptr, ptr::null::<GLvoid>(), gl_usage));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

		Box::new(GlIndexBuffer { generation: self.generation, buffer, size, _usage: usage, ty })
	}

	fn index_buffer_write(&mut self, buffer: &mut dyn crate::IndexBuffer, offset: usize, data: &[u8]) {
		let buf = objects::index_buffer_mut(buffer);
		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Index buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr() as *const _));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
	}

	fn shader_compile(&mut self, interface: &mut dyn crate::IShaderInterface, name: &str, defines: &[crate::ShaderDefine<'_>]) -> Box<dyn crate::ShaderProgram> {
		shader::compile2(self, interface, name, defines)
	}

	fn texture2d_create(&mut self, info: &crate::Texture2DInfo) -> Box<dyn crate::Texture2D> {
		texture2d::create(self, info)
	}

	fn texture2d_generate_mipmap(&mut self, texture: &mut dyn crate::Texture2D) {
		texture2d::generate_mipmap(self, texture)
	}

	fn texture2d_update(&mut self, texture: &mut dyn crate::Texture2D, info: &crate::Texture2DInfo) {
		texture2d::update(self, texture, info)
	}

	fn texture2d_write(&mut self, texture: &mut dyn crate::Texture2D, level: u8, data: &[u8]) {
		texture2d::write(self, texture, level, data)
	}

	fn texture2d_read_into(&mut self, texture: &dyn crate::Texture2D, level: u8, data: &mut [u8]) {
		texture2d::read_into(self, texture, level, data)
	}
}

impl ops::Deref for GlGraphics {
	type Target = dyn crate::IGraphics;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self
	}
}
impl ops::DerefMut for GlGraphics {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		self
	}
}
