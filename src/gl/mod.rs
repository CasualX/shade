/*!
OpenGL graphics backend.
*/

use std::{mem, ops, ptr, slice, time};
use std::any::type_name_of_val as name_of;
use std::collections::HashMap;

/// Re-exported OpenGL bindings.
pub use gl as capi;
use gl::types::*;

use crate::resources::{Resource, ResourceMap};

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

mod draw;
mod shader;
mod texture2d;

pub mod shaders;

fn gl_texture_wrap(wrap: crate::TextureWrap) -> GLint {
	(match wrap {
		crate::TextureWrap::Edge => gl::CLAMP_TO_EDGE,
		crate::TextureWrap::Border => gl::CLAMP_TO_BORDER,
		crate::TextureWrap::Repeat => gl::REPEAT,
		crate::TextureWrap::Mirror => gl::MIRRORED_REPEAT,
	}) as GLint
}
fn gl_texture_filter_mag(filter: crate::TextureFilter) -> GLint {
	(match filter {
		crate::TextureFilter::Nearest => gl::NEAREST,
		crate::TextureFilter::Linear => gl::LINEAR,
	}) as GLint
}

fn gl_texture_filter_min(props: &crate::TextureProps) -> GLint {
	(if props.mip_levels > 1 {
		match props.filter_min {
			crate::TextureFilter::Nearest => gl::NEAREST_MIPMAP_NEAREST,
			crate::TextureFilter::Linear => gl::LINEAR_MIPMAP_LINEAR,
		}
	}
	else {
		match props.filter_min {
			crate::TextureFilter::Nearest => gl::NEAREST,
			crate::TextureFilter::Linear => gl::LINEAR,
		}
	}) as GLint
}

struct GlVertexBuffer {
	buffer: GLuint,
	_size: usize,
	usage: crate::BufferUsage,
	layout: &'static crate::VertexLayout,
}
impl Resource for GlVertexBuffer {
	type Handle = crate::VertexBuffer;
}

struct GlIndexBuffer {
	buffer: GLuint,
	_size: usize,
	usage: crate::BufferUsage,
	ty: crate::IndexType,
}
impl Resource for GlIndexBuffer {
	type Handle = crate::IndexBuffer;
}

#[allow(dead_code)]
struct GlActiveAttrib {
	location: u32,
	size: GLint,
	ty: GLenum,
}

#[allow(dead_code)]
struct GlActiveUniform {
	location: GLint,
	size: GLint,
	ty: GLenum,
	texture_unit: i8, // Texture unit, -1 if not a sampler
}

struct GlShader {
	program: GLuint,
	attribs: HashMap<NameBuf, GlActiveAttrib>,
	uniforms: HashMap<NameBuf, GlActiveUniform>,
}
impl Resource for GlShader {
	type Handle = crate::Shader;
}

struct GlTexture2D {
	texture: GLuint,
	info: crate::Texture2DInfo,
}
impl Resource for GlTexture2D {
	type Handle = crate::Texture2D;
}

struct GlTextures {
	textures2d: ResourceMap<GlTexture2D>,
	textures2d_default: GlTexture2D,
}

impl GlTextures {
	fn get2d(&self, id: crate::Texture2D) -> &GlTexture2D {
		self.textures2d.get(id).unwrap_or(&self.textures2d_default)
	}
}

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
	vbuffers: ResourceMap<GlVertexBuffer>,
	ibuffers: ResourceMap<GlIndexBuffer>,
	shaders: ResourceMap<GlShader>,
	textures: GlTextures,
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

		let mut default_texture2d = 0;
		{
			gl_check!(gl::GenTextures(1, &mut default_texture2d));
			gl_check!(gl::BindTexture(gl::TEXTURE_2D, default_texture2d));
			let color = [0u8; 4];
			gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 1, 1, 0, gl::RGBA, gl::UNSIGNED_BYTE, color.as_ptr() as *const GLvoid));
			gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		}

		if config.srgb {
			gl_check!(gl::Enable(gl::FRAMEBUFFER_SRGB));
		}
		else {
			gl_check!(gl::Disable(gl::FRAMEBUFFER_SRGB));
		}

		GlGraphics {
			vbuffers: ResourceMap::new(),
			ibuffers: ResourceMap::new(),
			shaders: ResourceMap::new(),
			textures: GlTextures {
				textures2d: ResourceMap::new(),
				textures2d_default: GlTexture2D {
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
							border_color: [0.0, 0.0, 0.0, 0.0],
						}
					},
				},
			},
			dynamic_vao,
			drawing: false,
			draw_begin: time::Instant::now(),
			metrics: Default::default(),
			immediate_fbo: None,
			config,
		}
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

	fn vertex_buffer_create(&mut self, name: Option<&str>, size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> crate::VertexBuffer {
		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));

		let id = self.vbuffers.insert(name, GlVertexBuffer { buffer, _size: size, usage, layout });
		return id;
	}

	fn vertex_buffer_find(&self, name: &str) -> crate::VertexBuffer {
		self.vbuffers.find_id(name).unwrap_or(crate::VertexBuffer::INVALID)
	}

	fn vertex_buffer_write(&mut self, id: crate::VertexBuffer, data: &[u8]) {
		let Some(buf) = self.vbuffers.get_mut(id) else { return };
		let size = mem::size_of_val(data);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		let gl_usage = match buf.usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferData(gl::ARRAY_BUFFER, size as GLsizeiptr, data.as_ptr() as *const _, gl_usage));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
	}

	fn vertex_buffer_free(&mut self, id: crate::VertexBuffer, mode: crate::FreeMode) {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(buf) = self.vbuffers.remove(id) else { return };
		gl_check!(gl::DeleteBuffers(1, &buf.buffer));
	}

	fn index_buffer_create(&mut self, name: Option<&str>, size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> crate::IndexBuffer {
		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));

		let id = self.ibuffers.insert(name, GlIndexBuffer { buffer, _size: size, usage, ty });
		return id;
	}

	fn index_buffer_find(&self, name: &str) -> crate::IndexBuffer {
		self.ibuffers.find_id(name).unwrap_or(crate::IndexBuffer::INVALID)
	}

	fn index_buffer_write(&mut self, id: crate::IndexBuffer, data: &[u8]) {
		let Some(buf) = self.ibuffers.get_mut(id) else { return };
		let size = mem::size_of_val(data);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		let gl_usage = match buf.usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size as GLsizeiptr, data.as_ptr() as *const _, gl_usage));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
	}

	fn index_buffer_free(&mut self, id: crate::IndexBuffer, mode: crate::FreeMode) {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(buf) = self.ibuffers.remove(id) else { return };
		gl_check!(gl::DeleteBuffers(1, &buf.buffer));
	}

	fn shader_create(&mut self, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> crate::Shader {
		shader::create(self, name, vertex_source, fragment_source)
	}

	fn shader_find(&self, name: &str) -> crate::Shader {
		shader::find(self, name)
	}

	fn shader_free(&mut self, id: crate::Shader) {
		shader::delete(self, id)
	}

	fn texture2d_create(&mut self, name: Option<&str>, info: &crate::Texture2DInfo) -> crate::Texture2D {
		texture2d::create(self, name, info)
	}

	fn texture2d_find(&self, name: &str) -> crate::Texture2D {
		texture2d::find(self, name)
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

	fn texture2d_free(&mut self, id: crate::Texture2D, mode: crate::FreeMode) {
		texture2d::free(self, id, mode)
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
