//! WebGL graphics backend.

use std::{fmt, mem, slice};
use std::any::type_name_of_val as name_of;
use std::collections::HashMap;

mod api;
mod draw;
mod shader;

pub mod shaders;

use crate::resources::{Resource, ResourceMap};
use api::types::*;

type NameBuf = crate::sstring::SmallString<64>;

pub fn log(s: impl fmt::Display) {
	let s = s.to_string();
	unsafe { api::consoleLog(s.as_ptr(), s.len()) };
}

fn gl_texture_wrap(wrap: crate::TextureWrap) -> GLint {
	(match wrap {
		crate::TextureWrap::Edge => api::CLAMP_TO_EDGE,
		crate::TextureWrap::Border => unimplemented!("ClampBorder is not supported in WebGL"),
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

struct WebGLVertexBuffer {
	buffer: GLuint,
	_size: usize,
	usage: crate::BufferUsage,
	layout: &'static crate::VertexLayout,
}
impl Resource for WebGLVertexBuffer {
	type Handle = crate::VertexBuffer;
}

struct WebGLIndexBuffer {
	buffer: GLuint,
	_size: usize,
	usage: crate::BufferUsage,
	ty: crate::IndexType,
}
impl Resource for WebGLIndexBuffer {
	type Handle = crate::IndexBuffer;
}

#[allow(dead_code)]
struct WebGLActiveAttrib {
	location: GLuint,
	size: GLint,
	ty: GLenum,
}

#[allow(dead_code)]
struct WebGLActiveUniform {
	location: GLuint,
	size: GLint,
	ty: GLenum,
	texture_unit: i8, // -1 if not a texture
}

struct WebGLProgram {
	program: GLuint,
	// compile_log: String, // Displayed in JS console

	attribs: HashMap<NameBuf, WebGLActiveAttrib>,
	uniforms: HashMap<NameBuf, WebGLActiveUniform>,
}
impl Resource for WebGLProgram {
	type Handle = crate::Shader;
}

struct WebGLTexture2D {
	texture: GLuint,
	info: crate::Texture2DInfo,
}
impl Resource for WebGLTexture2D {
	type Handle = crate::Texture2D;
}

struct WebGLTextures {
	textures2d: ResourceMap<WebGLTexture2D>,
	textures2d_default: WebGLTexture2D,
}

impl WebGLTextures {
	fn get2d(&self, id: crate::Texture2D) -> &WebGLTexture2D {
		self.textures2d.get(id).unwrap_or(&self.textures2d_default)
	}
}

pub struct WebGLGraphics {
	vbuffers: ResourceMap<WebGLVertexBuffer>,
	ibuffers: ResourceMap<WebGLIndexBuffer>,
	shaders: ResourceMap<WebGLProgram>,
	textures: WebGLTextures,
	drawing: bool,
	metrics: crate::DrawMetrics,
}

impl WebGLGraphics {
	pub fn new() -> Self {

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

		WebGLGraphics {
			vbuffers: ResourceMap::new(),
			ibuffers: ResourceMap::new(),
			shaders: ResourceMap::new(),
			textures: WebGLTextures {
				textures2d: ResourceMap::new(),
				textures2d_default: WebGLTexture2D {
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
				},
			},
			drawing: false,
			metrics: Default::default(),
		}
	}
	/// Returns the graphics interface.
	#[inline]
	pub fn as_graphics(&mut self) -> &mut crate::Graphics {
		crate::Graphics(self)
	}
}

impl crate::IGraphics for WebGLGraphics {
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

	fn vertex_buffer_create(&mut self, name: Option<&str>, _size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> crate::VertexBuffer {
		let buffer = unsafe { api::createBuffer() };

		if layout.size >= 256 {
			panic!("Vertex layout size {} exceeds WebGL 1.0 limit of 256 bytes", layout.size);
		}

		let id = self.vbuffers.insert(name, WebGLVertexBuffer { buffer, _size, layout, usage });
		return id;
	}

	fn vertex_buffer_find(&self, name: &str) -> crate::VertexBuffer {
		self.vbuffers.find_id(name).unwrap_or(crate::VertexBuffer::INVALID)
	}

	fn vertex_buffer_write(&mut self, id: crate::VertexBuffer, data: &[u8]) {
		let Some(buf) = self.vbuffers.get_mut(id) else { return };

		let size = mem::size_of_val(data);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		let usage = match buf.usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferData(api::ARRAY_BUFFER, size as GLsizeiptr, data.as_ptr(), usage) };
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, 0) };
	}

	fn vertex_buffer_free(&mut self, id: crate::VertexBuffer, mode: crate::FreeMode) {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(vb) = self.vbuffers.remove(id) else { return };
		unsafe { api::deleteBuffer(vb.buffer) };
	}

	fn index_buffer_create(&mut self, name: Option<&str>, _size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> crate::IndexBuffer {
		let buffer = unsafe { api::createBuffer() };

		let id = self.ibuffers.insert(name, WebGLIndexBuffer { buffer, _size, usage, ty });
		return id;
	}

	fn index_buffer_find(&self, name: &str) -> crate::IndexBuffer {
		self.ibuffers.find_id(name).unwrap_or(crate::IndexBuffer::INVALID)
	}

	fn index_buffer_write(&mut self, id: crate::IndexBuffer, data: &[u8]) {
		let Some(buf) = self.ibuffers.get_mut(id) else { return };
		let size = mem::size_of_val(data);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, size);
		let usage = match buf.usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferData(api::ELEMENT_ARRAY_BUFFER, size as GLsizeiptr, data.as_ptr(), usage) };
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, 0) };
	}

	fn index_buffer_free(&mut self, id: crate::IndexBuffer, mode: crate::FreeMode) {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(vb) = self.ibuffers.remove(id) else { return };
		unsafe { api::deleteBuffer(vb.buffer) };
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
		let texture = unsafe { api::createTexture() };
		unsafe { api::bindTexture(api::TEXTURE_2D, texture) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)) };
		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
		let id = self.textures.textures2d.insert(name, WebGLTexture2D { texture, info: info.clone() });
		return id;
	}

	fn texture2d_find(&self, name: &str) -> crate::Texture2D {
		self.textures.textures2d.find_id(name).unwrap_or(crate::Texture2D::INVALID)
	}

	fn texture2d_generate_mipmap(&mut self, id: crate::Texture2D) {
		let Some(texture) = self.textures.textures2d.get(id) else { return };
		unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
		unsafe { api::generateMipmap(api::TEXTURE_2D) };
		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
	}

	fn texture2d_update(&mut self, id: crate::Texture2D, info: &crate::Texture2DInfo) -> crate::Texture2D {
		let Some(texture) = self.textures.textures2d.get_mut(id) else {
			return self.texture2d_create(None, info);
		};

		let realloc = texture.info.width != info.width ||
			texture.info.height != info.height ||
			texture.info.format != info.format ||
			texture.info.props.mip_levels != info.props.mip_levels;

		// WebGL 1 has no TexStorage2D. To "reallocate" we recreate the underlying
		// texture object and (re)define mip levels with texImage2D.
		if realloc {
			unsafe { api::deleteTexture(texture.texture) };
			texture.texture = unsafe { api::createTexture() };
		}

		unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)) };

		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
		texture.info = *info;
		return id;
	}

	fn texture2d_write(&mut self, id: crate::Texture2D, level: u8, data: &[u8]) {
		let Some(texture) = self.textures.textures2d.get(id) else { return };
		assert!(level < texture.info.props.mip_levels, "Invalid mip level {}", level);
		self.metrics.bytes_uploaded = usize::wrapping_add(self.metrics.bytes_uploaded, data.len());
		unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
		unsafe { api::pixelStorei(api::UNPACK_ALIGNMENT, 1) }; // Set unpack alignment to 1 byte
		let WebGLTextureFormat { internalformat, format, type_ } = WebGLTextureFormat::get(texture.info.format);
		unsafe { api::texImage2D(api::TEXTURE_2D, level as GLint, internalformat, texture.info.width, texture.info.height, 0, format, type_, data.as_ptr(), data.len()) };
		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
	}

	fn texture2d_read_into(&mut self, _id: crate::Texture2D, _level: u8, _data: &mut [u8]) {
		unimplemented!()
	}

	fn texture2d_get_info(&self, id: crate::Texture2D) -> Option<&crate::Texture2DInfo> {
		self.textures.textures2d.get(id).map(|texture| &texture.info)
	}

	fn texture2d_free(&mut self, id: crate::Texture2D, mode: crate::FreeMode) {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(texture) = self.textures.textures2d.remove(id) else { return };
		unsafe { api::deleteTexture(texture.texture) };
	}
}

struct WebGLTextureFormat {
	internalformat: GLenum,
	format: GLenum,
	type_: GLenum,
}

impl WebGLTextureFormat {
	fn get(format: crate::TextureFormat) -> WebGLTextureFormat {
		let (internalformat, format, type_) = match format {
			crate::TextureFormat::RGBA8 => (api::RGBA, api::RGBA, api::UNSIGNED_BYTE),
			crate::TextureFormat::RGB8 => (api::RGB, api::RGB, api::UNSIGNED_BYTE),
			crate::TextureFormat::R8 => (api::LUMINANCE, api::LUMINANCE, api::UNSIGNED_BYTE),
			_ => unimplemented!("Texture format {:?} is not supported in WebGL 1", format),
		};
		WebGLTextureFormat { internalformat, format, type_ }
	}
}
