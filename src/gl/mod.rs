/*!
OpenGL graphics backend.
*/

use std::{mem, ops, ptr, slice};

/// Re-exported OpenGL bindings.
pub use gl as capi;
use gl::types::*;

pub const MTSDF_FS: &str = include_str!("shaders/mtsdf.fs.glsl");
pub const MTSDF_VS: &str = include_str!("shaders/mtsdf.vs.glsl");

use crate::resources::{Resource, ResourceMap};
use crate::handle::Handle;

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

fn gl_texture_wrap(wrap: crate::TextureWrap) -> GLint {
	(match wrap {
		crate::TextureWrap::ClampEdge => gl::CLAMP_TO_EDGE,
		crate::TextureWrap::ClampBorder => gl::CLAMP_TO_BORDER,
		crate::TextureWrap::Repeat => gl::REPEAT,
		crate::TextureWrap::Mirror => gl::MIRRORED_REPEAT,
	}) as GLint
}
fn gl_texture_filter(filter: crate::TextureFilter) -> GLint {
	(match filter {
		crate::TextureFilter::Nearest => gl::NEAREST,
		crate::TextureFilter::Linear => gl::LINEAR,
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

struct GlActiveAttrib {
	location: GLint,
	namelen: u8,
	namebuf: [u8; 64],
	_size: GLint,
	_ty: GLenum,
}
impl GlActiveAttrib {
	fn name(&self) -> &str {
		str::from_utf8(&self.namebuf[..self.namelen as usize]).unwrap_or("err")
	}
}

struct GlActiveUniform {
	location: GLint,
	namelen: u8,
	namebuf: [u8; 64],
	_size: GLint,
	_ty: GLenum,
	texture_unit: i8, // Texture unit, -1 if not a sampler
}
impl GlActiveUniform {
	fn name(&self) -> &str {
		str::from_utf8(&self.namebuf[..self.namelen as usize]).unwrap_or("err")
	}
}

struct GlShader {
	program: GLuint,
	attribs: Vec<GlActiveAttrib>,
	uniforms: Vec<GlActiveUniform>,
}
impl Resource for GlShader {
	type Handle = crate::Shader;
}
impl GlShader {
	fn get_attrib(&self, name: &str) -> Option<&GlActiveAttrib> {
		self.attribs.iter().find(|a| a.name() == name)
	}
	fn get_uniform(&self, name: &str) -> Option<&GlActiveUniform> {
		self.uniforms.iter().find(|u| u.name() == name)
	}
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

#[allow(dead_code)]
struct GlSurface {
	texture: crate::Texture2D,
	frame_buf: GLuint,
	depth_buf: GLuint,
	tex_buf: GLuint,
	format: crate::SurfaceFormat,
	width: i32,
	height: i32,
}
impl Resource for GlSurface {
	type Handle = crate::Surface;
}

pub struct GlGraphics {
	vbuffers: ResourceMap<GlVertexBuffer>,
	ibuffers: ResourceMap<GlIndexBuffer>,
	shaders: ResourceMap<GlShader>,
	textures: GlTextures,
	surfaces: ResourceMap<GlSurface>,
	dynamic_vao: GLuint,
	drawing: bool,
}

impl GlGraphics {
	pub fn new() -> Self {
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
						filter_min: crate::TextureFilter::Nearest,
						filter_mag: crate::TextureFilter::Nearest,
						wrap_u: crate::TextureWrap::ClampEdge,
						wrap_v: crate::TextureWrap::ClampEdge,
						border_color: [0, 0, 0, 0],
					},
				},
			},
			surfaces: ResourceMap::new(),
			dynamic_vao,
			drawing: false,
		}
	}
}

impl crate::IGraphics for GlGraphics {
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

	fn vertex_buffer_create(&mut self, name: Option<&str>, size: usize, layout: &'static crate::VertexLayout, usage: crate::BufferUsage) -> Result<crate::VertexBuffer, crate::GfxError> {
		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));

		let id = self.vbuffers.insert(name, GlVertexBuffer { buffer, _size: size, usage, layout });
		return Ok(id);
	}

	fn vertex_buffer_find(&mut self, name: &str) -> Result<crate::VertexBuffer, crate::GfxError> {
		self.vbuffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn vertex_buffer_set_data(&mut self, id: crate::VertexBuffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.vbuffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as GLsizeiptr;
		let gl_usage = match buf.usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferData(gl::ARRAY_BUFFER, size, data.as_ptr() as *const _, gl_usage));
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
		Ok(())
	}

	fn vertex_buffer_free(&mut self, id: crate::VertexBuffer, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(buf) = self.vbuffers.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteBuffers(1, &buf.buffer));
		Ok(())
	}

	fn index_buffer_create(&mut self, name: Option<&str>, size: usize, ty: crate::IndexType, usage: crate::BufferUsage) -> Result<crate::IndexBuffer, crate::GfxError> {
		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));

		let id = self.ibuffers.insert(name, GlIndexBuffer { buffer, _size: size, usage, ty });
		return Ok(id);
	}

	fn index_buffer_find(&mut self, name: &str) -> Result<crate::IndexBuffer, crate::GfxError> {
		self.ibuffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn index_buffer_set_data(&mut self, id: crate::IndexBuffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.ibuffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as GLsizeiptr;
		let gl_usage = match buf.usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf.buffer));
		gl_check!(gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data.as_ptr() as *const _, gl_usage));
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
		Ok(())
	}

	fn index_buffer_free(&mut self, id: crate::IndexBuffer, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(buf) = self.ibuffers.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteBuffers(1, &buf.buffer));
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
		let mut texture = 0;
		gl_check!(gl::GenTextures(1, &mut texture));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl_texture_wrap(info.wrap_u)));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl_texture_wrap(info.wrap_v)));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_texture_filter(info.filter_mag)));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_texture_filter(info.filter_min)));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		let id = self.textures.textures2d.insert(name, GlTexture2D { texture, info: *info });
		return Ok(id);
	}

	fn texture2d_find(&mut self, name: &str) -> Result<crate::Texture2D, crate::GfxError> {
		self.textures.textures2d.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn texture2d_set_data(&mut self, id: crate::Texture2D, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
		gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
		let (internal_format, format) = match texture.info.format {
			crate::TextureFormat::RGB8 => (gl::RGB8 as GLint, gl::RGB),
			crate::TextureFormat::RGBA8 => (gl::RGBA8 as GLint, gl::RGBA),
			crate::TextureFormat::Grey8 => (gl::R8 as GLint, gl::RED),
		};
		gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format, texture.info.width, texture.info.height, 0, format, gl::UNSIGNED_BYTE, data.as_ptr() as *const _));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		Ok(())
	}

	fn texture2d_get_info(&mut self, id: crate::Texture2D) -> Result<crate::Texture2DInfo, crate::GfxError> {
		let Some(texture) = self.textures.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(texture.info);
	}

	fn texture2d_free(&mut self, id: crate::Texture2D, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(texture) = self.textures.textures2d.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteTextures(1, &texture.texture));
		Ok(())
	}

	// fn texture2darray_create(&mut self, name: Option<&str>, info: &crate::Texture2DArrayInfo) -> Result<crate::Texture2DArray, crate::GfxError> {
	// 	let mut texture = 0;
	// 	gl_check!(gl::GenTextures(1, &mut texture));
	// 	gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture));
	// 	gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl_texture_wrap(info.wrap_u)));
	// 	gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl_texture_wrap(info.wrap_v)));
	// 	gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl_texture_filter(info.filter_mag)));
	// 	gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl_texture_filter(info.filter_min)));
	// 	gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0));
	// 	let id = self.textures.textures2darray.insert(name, GlTexture2DArray { texture, info: *info });
	// 	return Ok(id);
	// }
	// fn texture2darray_find(&mut self, name: &str) -> Result<crate::Texture2DArray, crate::GfxError> {
	// 	self.textures.textures2darray.find_id(name).ok_or(crate::GfxError::NameNotFound)
	// }
	// fn texture2darray_set_data(&mut self, id: crate::Texture2DArray, index: usize, data: &[u8]) -> Result<(), crate::GfxError> {
	// 	let Some(texture) = self.textures.textures2darray.get(id) else { return Err(crate::GfxError::InvalidHandle) };
	// 	if index >= texture.info.count as usize { return Err(crate::GfxError::IndexOutOfBounds) }
	// 	gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture.texture));
	// 	let (internal_format, format) = match texture.info.format {
	// 		crate::TextureFormat::RGB8 => (gl::RGB8 as GLint, gl::RGB),
	// 		crate::TextureFormat::RGBA8 => (gl::RGBA8 as GLint, gl::RGBA),
	// 		crate::TextureFormat::Grey8 => (gl::R8 as GLint, gl::RED),
	// 	};
	// 	gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
	// 	gl_check!(gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, internal_format, texture.info.width, texture.info.height, index as i32, 0, format, gl::UNSIGNED_BYTE, data.as_ptr() as *const _));
	// 	gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0));
	// 	Ok(())
	// }
	// fn texture2darray_get_info(&mut self, id: crate::Texture2DArray) -> Result<crate::Texture2DArrayInfo, crate::GfxError> {
	// 	let Some(texture) = self.textures.textures2darray.get(id) else { return Err(crate::GfxError::InvalidHandle) };
	// 	return Ok(texture.info);
	// }
	// fn texture2darray_free(&mut self, id: crate::Texture2DArray, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
	// 	assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
	// 	let Some(texture) = self.textures.textures2darray.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
	// 	gl_check!(gl::DeleteTextures(1, &texture.texture));
	// 	Ok(())
	// }

	fn surface_create(&mut self, name: Option<&str>, info: &crate::SurfaceInfo) -> Result<crate::Surface, crate::GfxError> {
		let texture = Handle::create(0);

		let mut frame_buf = 0;
		let mut depth_buf = 0;
		let mut tex_buf = 0;
		gl_check!(gl::GenFramebuffers(1, &mut frame_buf));
		gl_check!(gl::GenRenderbuffers(1, &mut depth_buf));
		gl_check!(gl::GenTextures(1, &mut tex_buf));

		gl_check!(gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buf));

		gl_check!(gl::BindRenderbuffer(gl::RENDERBUFFER, depth_buf));
		gl_check!(gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, info.width, info.height));
		gl_check!(gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, depth_buf));

		gl_check!(gl::BindTexture(gl::TEXTURE_2D, tex_buf));

		let format = match info.format {
			crate::SurfaceFormat::RGB8 => gl::RGB,
			crate::SurfaceFormat::RGBA8 => gl::RGBA,
		};
		gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
		gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, info.width, info.height, 0, format, gl::UNSIGNED_BYTE, ptr::null::<GLvoid>()));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32));

		gl_check!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_buf, 0));

		gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		gl_check!(gl::BindRenderbuffer(gl::RENDERBUFFER, 0));
		gl_check!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));

		// let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
		// if status != gl::FRAMEBUFFER_COMPLETE {
		// 	panic!("Framebuffer is not complete: {}", status);
		// }

		let id = self.surfaces.insert(name, GlSurface { texture, frame_buf, depth_buf, tex_buf, format: info.format, width: info.width, height: info.height });
		return Ok(id);
	}

	fn surface_find(&mut self, name: &str) -> Result<crate::Surface, crate::GfxError> {
		self.surfaces.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn surface_get_info(&mut self, id: crate::Surface) -> Result<crate::SurfaceInfo, crate::GfxError> {
		let Some(surface) = self.surfaces.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(crate::SurfaceInfo {
			offscreen: true,
			has_depth: surface.depth_buf != 0,
			has_texture: surface.texture.id() != 0,
			format: surface.format,
			width: surface.width,
			height: surface.height,
		});
	}

	fn surface_set_info(&mut self, _id: crate::Surface, _info: &crate::SurfaceInfo) -> Result<(), crate::GfxError> {
		Err(crate::GfxError::InternalError)
	}

	fn surface_get_texture(&mut self, id: crate::Surface) -> Result<crate::Texture2D, crate::GfxError> {
		let Some(surface) = self.surfaces.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(surface.texture);
	}

	fn surface_free(&mut self, id: crate::Surface, mode: crate::FreeMode) -> Result<(), crate::GfxError> {
		assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
		let Some(surface) = self.surfaces.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
		self.texture2d_free(surface.texture, mode)?;
		Ok(())
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
