use super::*;
use std::collections::HashMap;

pub struct WebGLVertexBuffer {
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub layout: &'static crate::VertexLayout,
}
impl crate::Resource for WebGLVertexBuffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebGLVertexBuffer({:p})", self)
	}
}
impl crate::VertexBuffer for WebGLVertexBuffer {
	fn layout(&self) -> &'static crate::VertexLayout { self.layout }
}
impl Drop for WebGLVertexBuffer {
	fn drop(&mut self) {
		unsafe { api::deleteBuffer(self.buffer) };
	}
}

pub struct WebGLIndexBuffer {
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub ty: crate::IndexType,
}
impl crate::Resource for WebGLIndexBuffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebGLIndexBuffer({:p})", self)
	}
}
impl crate::IndexBuffer for WebGLIndexBuffer {
	fn index_type(&self) -> crate::IndexType { self.ty }
}
impl Drop for WebGLIndexBuffer {
	fn drop(&mut self) {
		unsafe { api::deleteBuffer(self.buffer) };
	}
}

#[allow(dead_code)]
pub struct WebGLActiveAttrib {
	pub location: GLuint,
	pub size: GLint,
	pub ty: GLenum,
}

#[allow(dead_code)]
pub struct WebGLActiveUniform {
	pub location: GLuint,
	pub size: GLint,
	pub ty: GLenum,
	pub texture_unit: i8,
}

pub struct WebGLShaderProgram {
	pub program: GLuint,
	pub attribs: HashMap<NameBuf, WebGLActiveAttrib>,
	pub uniforms: HashMap<NameBuf, WebGLActiveUniform>,
}
impl crate::Resource for WebGLShaderProgram {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebGLShaderProgram({:p})", self)
	}
}
impl crate::ShaderProgram for WebGLShaderProgram {
}
impl Drop for WebGLShaderProgram {
	fn drop(&mut self) {
		unsafe { api::deleteProgram(self.program) };
	}
}

pub struct WebGLTexture2D {
	pub texture: GLuint,
	pub info: crate::Texture2DInfo,
}
impl crate::Resource for WebGLTexture2D {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebGLTexture2D({:p})", self)
	}
}
impl crate::Texture2D for WebGLTexture2D {
	fn info(&self) -> &crate::Texture2DInfo { &self.info }
}
impl Drop for WebGLTexture2D {
	fn drop(&mut self) {
		unsafe { api::deleteTexture(self.texture) };
	}
}

pub struct WebGLTextureRef<'a> {
	pub texture: GLuint,
	pub info: &'a crate::Texture2DInfo,
}

#[track_caller]
pub fn vertex_buffer(buffer: &dyn crate::VertexBuffer) -> &WebGLVertexBuffer {
	let buffer: &dyn std::any::Any = buffer;
	buffer.downcast_ref::<WebGLVertexBuffer>().expect("WebGL backend received a foreign VertexBuffer")
}

#[track_caller]
pub fn vertex_buffer_mut(buffer: &mut dyn crate::VertexBuffer) -> &mut WebGLVertexBuffer {
	let buffer: &mut dyn std::any::Any = buffer;
	buffer.downcast_mut::<WebGLVertexBuffer>().expect("WebGL backend received a foreign VertexBuffer")
}

#[track_caller]
pub fn index_buffer(buffer: &dyn crate::IndexBuffer) -> &WebGLIndexBuffer {
	let buffer: &dyn std::any::Any = buffer;
	buffer.downcast_ref::<WebGLIndexBuffer>().expect("WebGL backend received a foreign IndexBuffer")
}

#[track_caller]
pub fn index_buffer_mut(buffer: &mut dyn crate::IndexBuffer) -> &mut WebGLIndexBuffer {
	let buffer: &mut dyn std::any::Any = buffer;
	buffer.downcast_mut::<WebGLIndexBuffer>().expect("WebGL backend received a foreign IndexBuffer")
}

#[track_caller]
pub fn shader_program(shader: &dyn crate::ShaderProgram) -> &WebGLShaderProgram {
	let shader: &dyn std::any::Any = shader;
	shader.downcast_ref::<WebGLShaderProgram>().expect("WebGL backend received a foreign ShaderProgram")
}

#[track_caller]
pub fn texture2d<'a>(this: &'a WebGLGraphics, texture: &'a dyn crate::Texture2D) -> WebGLTextureRef<'a> {
	let texture_any: &dyn std::any::Any = texture;
	if let Some(texture) = texture_any.downcast_ref::<WebGLTexture2D>() {
		return WebGLTextureRef { texture: texture.texture, info: &texture.info };
	}

	if !texture_any.is::<crate::DefaultTexture2D>() {
		panic!("WebGL backend received a foreign Texture2D");
	}

	WebGLTextureRef {
		texture: this.texture2d_default,
		info: &crate::DefaultTexture2D::INFO,
	}
}

#[track_caller]
pub fn texture2d_mut(texture: &mut dyn crate::Texture2D) -> &mut WebGLTexture2D {
	let texture: &mut dyn std::any::Any = texture;
	assert!(!texture.is::<crate::DefaultTexture2D>(), "WebGL backend cannot mutate the shared DefaultTexture2D");
	texture.downcast_mut::<WebGLTexture2D>().expect("WebGL backend received a foreign Texture2D")
}
