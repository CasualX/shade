use std::{fmt, mem, slice};

mod api;
mod draw;
mod shader;

use crate::resources::{Resource, ResourceMap};
use api::types::*;

pub fn log(s: impl fmt::Display) {
	let s = s.to_string();
	unsafe { api::consoleLog(s.as_ptr(), s.len()) };
}

fn gl_texture_wrap(wrap: crate::TextureWrap) -> GLint {
	(match wrap {
		crate::TextureWrap::ClampEdge => api::CLAMP_TO_EDGE,
		crate::TextureWrap::ClampBorder => unimplemented!("ClampBorder is not supported in WebGL"),
		crate::TextureWrap::Repeat => api::REPEAT,
		crate::TextureWrap::Mirror => api::MIRRORED_REPEAT,
	}) as GLint
}
fn gl_texture_filter(filter: crate::TextureFilter) -> GLint {
	(match filter {
		crate::TextureFilter::Nearest => api::NEAREST,
		crate::TextureFilter::Linear => api::LINEAR,
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

struct WebGLActiveUniform {
	location: GLint,
	namelen: u8,
	namebuf: [u8; 64],
	_size: GLint,
	_ty: GLenum,
	texture_unit: i8, // -1 if not a texture
}
impl WebGLActiveUniform {
	fn name(&self) -> &str {
		str::from_utf8(&self.namebuf[..self.namelen as usize]).unwrap_or("err")
	}
}

struct WebGLProgram {
	program: GLuint,
	// compile_log: String, // Displayed in JS console

	uniforms: Vec<WebGLActiveUniform>,
}
impl WebGLProgram {
	// fn get_attrib(&self, name: &str) -> Option<&WebGLActiveAttrib> {
	// 	self.attribs.iter().find(|a| a.name() == name)
	// }
	fn get_uniform(&self, name: &str) -> Option<&WebGLActiveUniform> {
		self.uniforms.iter().find(|u| u.name() == name)
	}
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

#[allow(dead_code)]
struct WebGLSurface {
	texture: crate::Texture2D,
	frame_buf: GLuint,
	depth_buf: GLuint,
	tex_buf: GLuint,
	format: crate::SurfaceFormat,
	width: i32,
	height: i32,
}

impl Resource for WebGLSurface {
	type Handle = crate::Surface;
}

pub struct WebGLGraphics {
	vbuffers: ResourceMap<WebGLVertexBuffer>,
	ibuffers: ResourceMap<WebGLIndexBuffer>,
	shaders: ResourceMap<WebGLProgram>,
	textures: WebGLTextures,
	surfaces: ResourceMap<WebGLSurface>,
	drawing: bool,
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
						filter_min: crate::TextureFilter::Nearest,
						filter_mag: crate::TextureFilter::Nearest,
						wrap_u: crate::TextureWrap::ClampEdge,
						wrap_v: crate::TextureWrap::ClampEdge,
						border_color: [0; 4],
					},
				},
			},
			surfaces: ResourceMap::new(),
			drawing: false,
		}
	}
}

impl crate::IGraphics for WebGLGraphics {
	fn begin(&mut self) -> Result<(), crate::GfxError> {
		draw::begin(self)
	}

	fn clear(&mut self, args: &crate::ClearArgs) -> Result<(), crate::GfxError> {
		draw::clear(self, args)
	}

	fn draw(&mut self, args: &crate::DrawArgs) -> Result<(), crate::GfxError> {
		draw::arrays(self, args)
	}

	fn draw_indexed(&mut self, args: &crate::DrawIndexedArgs) -> Result<(), crate::GfxError> {
		draw::indexed(self, args)
	}

	fn end(&mut self) -> Result<(), crate::GfxError> {
		draw::end(self)
	}

	fn vertex_buffer_create(&mut self, name: Option<&str>, _size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> Result<crate::VertexBuffer, crate::GfxError> {
		let buffer = unsafe { api::createBuffer() };

		let id = self.vbuffers.insert(name, WebGLVertexBuffer { buffer, _size, layout, usage });
		return Ok(id);
	}

	fn vertex_buffer_find(&mut self, name: &str) -> Result<crate::VertexBuffer, crate::GfxError> {
		self.vbuffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn vertex_buffer_set_data(&mut self, id: crate::VertexBuffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.vbuffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as GLsizeiptr;
		let usage = match buf.usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferData(api::ARRAY_BUFFER, size, data.as_ptr(), usage) };
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, 0) };
		Ok(())
	}

	fn vertex_buffer_free(&mut self, id: crate::VertexBuffer, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(vb) = self.vbuffers.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		unsafe { api::deleteBuffer(vb.buffer) };
		Ok(())
	}

	fn index_buffer_create(&mut self, name: Option<&str>, _size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> Result<crate::IndexBuffer, crate::GfxError> {
		let buffer = unsafe { api::createBuffer() };

		let id = self.ibuffers.insert(name, WebGLIndexBuffer { buffer, _size, usage, ty });
		return Ok(id);
	}

	fn index_buffer_find(&mut self, name: &str) -> Result<crate::IndexBuffer, crate::GfxError> {
		self.ibuffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn index_buffer_set_data(&mut self, id: crate::IndexBuffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.ibuffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as GLsizeiptr;
		let usage = match buf.usage {
			crate::BufferUsage::Static => api::STATIC_DRAW,
			crate::BufferUsage::Dynamic => api::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => api::STREAM_DRAW,
		};
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, buf.buffer) };
		unsafe { api::bufferData(api::ELEMENT_ARRAY_BUFFER, size, data.as_ptr(), usage) };
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, 0) };
		Ok(())
	}

	fn index_buffer_free(&mut self, id: crate::IndexBuffer, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(vb) = self.ibuffers.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		unsafe { api::deleteBuffer(vb.buffer) };
		Ok(())
	}

	fn shader_create(&mut self, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Result<crate::Shader, crate::GfxError> {
		shader::create(self, name, vertex_source, fragment_source)
	}

	fn shader_find(&mut self, name: &str) -> Result<crate::Shader, crate::GfxError> {
		shader::find(self, name)
	}

	fn shader_free(&mut self, id: crate::Shader) -> Result<(), crate::GfxError> {
		shader::delete(self, id)
	}

	fn texture2d_create(&mut self, name: Option<&str>, info: &crate::Texture2DInfo) -> Result<crate::Texture2D, crate::GfxError> {
		let texture = unsafe { api::createTexture() };
		unsafe { api::bindTexture(api::TEXTURE_2D, texture) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, gl_texture_wrap(info.wrap_u)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, gl_texture_wrap(info.wrap_v)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, gl_texture_filter(info.filter_min)) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, gl_texture_filter(info.filter_mag)) };
		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
		let id = self.textures.textures2d.insert(name, WebGLTexture2D { texture, info: info.clone() });
		return Ok(id);
	}

	fn texture2d_find(&mut self, name: &str) -> Result<crate::Texture2D, crate::GfxError> {
		self.textures.textures2d.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn texture2d_set_data(&mut self, id: crate::Texture2D, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
		unsafe { api::pixelStorei(api::UNPACK_ALIGNMENT, 1) }; // Set unpack alignment to 1 byte
		let format = match texture.info.format {
			crate::TextureFormat::RGB8 => api::RGB,
			crate::TextureFormat::RGBA8 => api::RGBA,
			crate::TextureFormat::Grey8 => api::LUMINANCE,
		};
		unsafe { api::texImage2D(api::TEXTURE_2D, 0, format, texture.info.width, texture.info.height, 0, format, api::UNSIGNED_BYTE, data.as_ptr(), data.len()) };
		unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
		Ok(())
	}

	fn texture2d_get_info(&mut self, id: crate::Texture2D) -> Result<crate::Texture2DInfo, crate::GfxError> {
		let Some(texture) = self.textures.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(texture.info);
	}

	fn texture2d_free(&mut self, id: crate::Texture2D, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(texture) = self.textures.textures2d.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		unsafe { api::deleteTexture(texture.texture) };
		Ok(())
	}

	fn surface_create(&mut self, name: Option<&str>, info: &crate::SurfaceInfo) -> Result<crate::Surface, crate::GfxError> {
		todo!()
	}

	fn surface_find(&mut self, name: &str) -> Result<crate::Surface, crate::GfxError> {
		todo!()
	}

	fn surface_get_info(&mut self, id: crate::Surface) -> Result<crate::SurfaceInfo, crate::GfxError> {
		todo!()
	}

	fn surface_set_info(&mut self, id: crate::Surface, info: &crate::SurfaceInfo) -> Result<(), crate::GfxError> {
		todo!()
	}

	fn surface_get_texture(&mut self, id: crate::Surface) -> Result<crate::Texture2D, crate::GfxError> {
		todo!()
	}

	fn surface_free(&mut self, id: crate::Surface, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		todo!()
	}

}
