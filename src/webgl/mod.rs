//! WebGL graphics backend.

use std::{fmt, mem, ops, panic, slice, time};
use std::any::type_name_of_val as name_of;
use std::collections::HashMap;

mod api;
mod draw;
mod shader;
mod texture2d;
mod objects;

pub mod shaders;

use self::objects::*;
use api::types::*;

type NameBuf = crate::sstring::SmallString<64>;

pub fn log(s: impl fmt::Display) {
	let s = s.to_string();
	unsafe { api::consoleLog(s.as_ptr(), s.len()) };
}

/// Sets up panic hook to log panics to the JavaScript console.
pub fn setup_panic_hook() {
	panic::set_hook(Box::new(|info| log(info)));
}

fn gl_texture_wrap(wrap: crate::TextureWrap) -> GLint {
	(match wrap {
		crate::TextureWrap::Edge => api::CLAMP_TO_EDGE,
		crate::TextureWrap::Border => api::CLAMP_TO_EDGE, // CLAMP_TO_BORDER not supported in WebGL 2.0
		crate::TextureWrap::Repeat => api::REPEAT,
		crate::TextureWrap::Mirror => api::MIRRORED_REPEAT,
	}) as GLint
}
fn gl_texture_filter_mag(filter: crate::TextureFilter) -> GLint {
	(match filter {
		crate::TextureFilter::Nearest => api::NEAREST,
		crate::TextureFilter::Linear => api::LINEAR,
	}) as GLint
}

fn gl_texture_filter_min(props: &crate::TextureProps) -> GLint {
	(if props.mip_levels > 1 {
		match props.filter_min {
			crate::TextureFilter::Nearest => api::NEAREST_MIPMAP_NEAREST,
			crate::TextureFilter::Linear => api::LINEAR_MIPMAP_LINEAR,
		}
	}
	else {
		match props.filter_min {
			crate::TextureFilter::Nearest => api::NEAREST,
			crate::TextureFilter::Linear => api::LINEAR,
		}
	}) as GLint
}

fn gl_depth_func(v: crate::Compare) -> GLenum {
	match v {
		crate::Compare::Never => api::NEVER,
		crate::Compare::Less => api::LESS,
		crate::Compare::Equal => api::EQUAL,
		crate::Compare::LessEqual => api::LEQUAL,
		crate::Compare::Greater => api::GREATER,
		crate::Compare::NotEqual => api::NOTEQUAL,
		crate::Compare::GreaterEqual => api::GEQUAL,
		crate::Compare::Always => api::ALWAYS,
	}
}

#[derive(Clone, Debug)]
pub struct WebGLConfig {
	pub srgb: bool,
}
impl Default for WebGLConfig {
	fn default() -> Self {
		WebGLConfig {
			srgb: true,
		}
	}
}

pub struct WebGLGraphics {
	objects: ObjectMap,
	texture2d_default: crate::Texture2D,
	_config: WebGLConfig,
	drawing: bool,
	draw_begin: f64,
	metrics: crate::DrawMetrics,
	/// Temporary framebuffer for immediate render passes, deleted at end().
	immediate_fbo: Option<GLuint>,
}

impl WebGLGraphics {
	pub fn new(config: WebGLConfig) -> Self {

		let default_texture2d;
		unsafe {
			default_texture2d = api::createTexture();
			api::bindTexture(api::TEXTURE_2D, default_texture2d);
			api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, api::CLAMP_TO_EDGE as GLint);
			api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, api::CLAMP_TO_EDGE as GLint);
			api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, api::NEAREST as GLint);
			api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, api::NEAREST as GLint);
			let data = [0u8; 4];
			api::texImage2D(api::TEXTURE_2D, 0, api::RGBA, 1, 1, 0, api::RGBA, api::UNSIGNED_BYTE, data.as_ptr() as *const _, 4);
			api::bindTexture(api::TEXTURE_2D, 0);
		}

		let mut objects = ObjectMap::new();
		let texture2d_default = objects.insert(WebGLTexture2D {
			texture: default_texture2d,
			info: crate::Texture2DInfo {
				width: 1,
				height: 1,
				format: crate::TextureFormat::RGBA8,
				props: crate::TextureProps {
					mip_levels: 1,
					usage: crate::TextureUsage::TEXTURE,
					filter_min: crate::TextureFilter::Nearest,
					filter_mag: crate::TextureFilter::Nearest,
					wrap_u: crate::TextureWrap::Edge,
					wrap_v: crate::TextureWrap::Edge,
					..Default::default()
				},
			},
		});

		WebGLGraphics {
			objects,
			texture2d_default,
			_config: config,
			drawing: false,
			draw_begin: 0.0,
			metrics: Default::default(),
			immediate_fbo: None,
		}
	}
	/// Returns the graphics interface.
	#[inline]
	pub fn as_graphics(&mut self) -> &mut crate::Graphics {
		crate::Graphics(self)
	}
}

impl crate::IGraphics for WebGLGraphics {
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

		if layout.size >= 256 {
			panic!("Vertex layout size {} exceeds WebGL 1.0 limit of 256 bytes", layout.size);
		}

		let buffer = unsafe { api::createBuffer() };
		let usage_enum = match usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};

		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buffer) };
		unsafe { api::bufferData(api::ARRAY_BUFFER, alloc_size as GLsizeiptr, std::ptr::null(), usage_enum) };
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, 0) };

		return self.objects.insert(WebGLVertexBuffer { buffer, size, layout, _usage: usage });
	}

	fn vertex_buffer_write(&mut self, id: crate::VertexBuffer, offset: usize, data: &[u8]) {
		let Some(buf) = self.objects.get_vertex_buffer(id) else { return };

		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Vertex buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferSubData(api::ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr()) };
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, 0) };
	}

	fn index_buffer_create(&mut self, size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> crate::IndexBuffer {
		// Ensure non-zero size? What to do with zero sized buffers?
		let alloc_size = if size == 0 { 1 } else { size };

		let buffer = unsafe { api::createBuffer() };
		let usage_enum = match usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};

		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, buffer) };
		unsafe { api::bufferData(api::ELEMENT_ARRAY_BUFFER, alloc_size as GLsizeiptr, std::ptr::null(), usage_enum) };
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, 0) };

		return self.objects.insert(WebGLIndexBuffer { buffer, size, _usage: usage, ty });
	}

	fn index_buffer_write(&mut self, id: crate::IndexBuffer, offset: usize, data: &[u8]) {
		let Some(buf) = self.objects.get_index_buffer(id) else { return };
		let size = mem::size_of_val(data);
		debug_assert!(offset + size <= buf.size, "Index buffer write out of bounds: {}..{} > {}", offset, offset + size, buf.size);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferSubData(api::ELEMENT_ARRAY_BUFFER, offset as GLintptr, size as GLsizeiptr, data.as_ptr()) };
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, 0) };
	}

	fn shader_compile(&mut self, vertex_source: &str, fragment_source: &str) -> crate::ShaderProgram {
		shader::create(self, vertex_source, fragment_source)
	}

	fn texture2d_create(&mut self, info: &crate::Texture2DInfo) -> crate::Texture2D {
		texture2d::create(self, info)
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

	fn texture2d_get_info(&self, id: crate::Texture2D) -> Option<&crate::Texture2DInfo> {
		texture2d::get_info(self, id)
	}
}

impl ops::Deref for WebGLGraphics {
	type Target = crate::Graphics;

	#[inline]
	fn deref(&self) -> &crate::Graphics {
		unsafe { mem::transmute(self as &dyn crate::IGraphics) }
	}
}
impl ops::DerefMut for WebGLGraphics {
	#[inline]
	fn deref_mut(&mut self) -> &mut crate::Graphics {
		crate::Graphics(self)
	}
}
