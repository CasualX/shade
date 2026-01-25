use super::*;
use std::collections::HashMap;

pub trait WebGLObjectTrait {
	type Handle;

	fn create_handle(value: u32) -> Self::Handle;

	fn stuff(self) -> WebGLObjectType;
}

pub struct WebGLVertexBuffer {
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub layout: &'static crate::VertexLayout,
}
impl WebGLObjectTrait for WebGLVertexBuffer {
	type Handle = crate::VertexBuffer;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::VertexBuffer { value }
	}
	#[inline]
	fn stuff(self) -> WebGLObjectType {
		WebGLObjectType::VertexBuffer(self)
	}
}

pub struct WebGLIndexBuffer {
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub ty: crate::IndexType,
}
impl WebGLObjectTrait for WebGLIndexBuffer {
	type Handle = crate::IndexBuffer;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::IndexBuffer { value }
	}
	#[inline]
	fn stuff(self) -> WebGLObjectType {
		WebGLObjectType::IndexBuffer(self)
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
	pub texture_unit: i8, // -1 if not a texture
}

pub struct WebGLShaderProgram {
	pub program: GLuint,
	// compile_log: String, // Displayed in JS console

	pub attribs: HashMap<NameBuf, WebGLActiveAttrib>,
	pub uniforms: HashMap<NameBuf, WebGLActiveUniform>,
}
impl WebGLObjectTrait for WebGLShaderProgram {
	type Handle = crate::ShaderProgram;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::ShaderProgram { value }
	}
	#[inline]
	fn stuff(self) -> WebGLObjectType {
		WebGLObjectType::ShaderProgram(self)
	}
}

pub struct WebGLTexture2D {
	pub texture: GLuint,
	pub info: crate::Texture2DInfo,
}
impl WebGLObjectTrait for WebGLTexture2D {
	type Handle = crate::Texture2D;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::Texture2D { value }
	}
	#[inline]
	fn stuff(self) -> WebGLObjectType {
		WebGLObjectType::Texture2D(self)
	}
}

pub struct WebGLObject {
	pub ref_count: u32,
	pub obj: WebGLObjectType,
}
impl WebGLObject {
	pub fn release(self) {
		match self.obj {
			WebGLObjectType::VertexBuffer(buf) => {
				unsafe { api::deleteBuffer(buf.buffer) };
			}
			WebGLObjectType::IndexBuffer(buf) => {
				unsafe { api::deleteBuffer(buf.buffer) };
			}
			WebGLObjectType::ShaderProgram(shader) => {
				shader::release(&shader);
			}
			WebGLObjectType::Texture2D(texture) => {
				texture2d::release(&texture);
			}
		}
	}
}

pub enum WebGLObjectType {
	VertexBuffer(WebGLVertexBuffer),
	IndexBuffer(WebGLIndexBuffer),
	ShaderProgram(WebGLShaderProgram),
	Texture2D(WebGLTexture2D),
}

pub struct ObjectMap {
	objects: HashMap<u32, WebGLObject>,
	last: u32,
}
impl ObjectMap {
	pub fn new() -> ObjectMap {
		ObjectMap {
			objects: HashMap::new(),
			last: 0,
		}
	}
	pub fn get_type(&self, handle: crate::BaseObject) -> Option<crate::ObjectType> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		match &object.obj {
			WebGLObjectType::VertexBuffer(_) => Some(crate::ObjectType::VertexBuffer),
			WebGLObjectType::IndexBuffer(_) => Some(crate::ObjectType::IndexBuffer),
			WebGLObjectType::ShaderProgram(_) => Some(crate::ObjectType::ShaderProgram),
			WebGLObjectType::Texture2D(_) => Some(crate::ObjectType::Texture2D),
		}
	}
	#[track_caller]
	pub fn add_ref(&mut self, handle: crate::BaseObject) {
		if handle == crate::BaseObject::INVALID {
			return;
		}
		let obj = self.objects.get_mut(&handle.value).expect("Invalid object handle");
		obj.ref_count += 1;
	}
	#[track_caller]
	pub fn release(&mut self, handle: crate::BaseObject) -> u32 {
		if handle == crate::BaseObject::INVALID {
			return 0;
		}
		let Some(obj) = self.objects.get_mut(&handle.value) else {
			return 0;
		};
		if obj.ref_count > 1 {
			obj.ref_count -= 1;
			return obj.ref_count;
		}
		if let Some(obj) = self.objects.remove(&handle.value) {
			obj.release();
		}
		return 0;
	}

	pub fn insert<T: WebGLObjectTrait>(&mut self, obj: T) -> T::Handle {
		loop {
			self.last = self.last.wrapping_add(1);
			if !self.objects.contains_key(&self.last) {
				break;
			}
		}
		let value = self.last;
		self.objects.insert(value, WebGLObject { ref_count: 1, obj: obj.stuff() });
		T::create_handle(value)
	}
}
impl ObjectMap {
	pub fn get_vertex_buffer(&self, handle: crate::VertexBuffer) -> Option<&WebGLVertexBuffer> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let WebGLObjectType::VertexBuffer(ref buf) = object.obj else { return None };
		return Some(buf);
	}
	pub fn get_index_buffer(&self, handle: crate::IndexBuffer) -> Option<&WebGLIndexBuffer> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let WebGLObjectType::IndexBuffer(ref buf) = object.obj else { return None };
		return Some(buf);
	}
	pub fn get_shader_program(&self, handle: crate::ShaderProgram) -> Option<&WebGLShaderProgram> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let WebGLObjectType::ShaderProgram(ref shader) = object.obj else { return None };
		return Some(shader);
	}
	pub fn get_texture2d(&self, handle: crate::Texture2D) -> Option<&WebGLTexture2D> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let WebGLObjectType::Texture2D(ref texture) = object.obj else { return None };
		return Some(texture);
	}
	pub fn get_texture2d_mut(&mut self, handle: crate::Texture2D) -> Option<&mut WebGLTexture2D> {
		let Some(object) = self.objects.get_mut(&handle.value) else { return None };
		let WebGLObjectType::Texture2D(ref mut texture) = object.obj else { return None };
		return Some(texture);
	}
}
