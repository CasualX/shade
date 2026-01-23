use super::*;

pub trait GlObjectTrait {
	type Handle;

	fn create_handle(value: u32) -> Self::Handle;

	fn stuff(self) -> GlObjectType;
}


pub struct GlVertexBuffer {
	pub buffer: GLuint,
	pub _size: usize,
	pub usage: crate::BufferUsage,
	pub layout: &'static crate::VertexLayout,
}
impl GlObjectTrait for GlVertexBuffer {
	type Handle = crate::VertexBuffer;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::VertexBuffer { value }
	}
	#[inline]
	fn stuff(self) -> GlObjectType {
		GlObjectType::VertexBuffer(self)
	}
}


pub struct GlIndexBuffer {
	pub buffer: GLuint,
	pub _size: usize,
	pub usage: crate::BufferUsage,
	pub ty: crate::IndexType,
}
impl GlObjectTrait for GlIndexBuffer {
	type Handle = crate::IndexBuffer;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::IndexBuffer { value }
	}
	#[inline]
	fn stuff(self) -> GlObjectType {
		GlObjectType::IndexBuffer(self)
	}
}


#[allow(dead_code)]
pub struct GlActiveAttrib {
	pub location: u32,
	pub size: GLint,
	pub ty: GLenum,
}
pub struct GlActiveUniform {
	pub location: GLint,
	pub array_size: GLint, // Number of elements in array, 1 if not an array
	pub ty: GLenum,
	pub texture_unit: i8, // Texture unit, -1 if not a sampler
}
pub struct GlShaderProgram {
	pub program: GLuint,
	pub attribs: HashMap<NameBuf, GlActiveAttrib>,
	pub uniforms: HashMap<NameBuf, GlActiveUniform>,
}
impl GlObjectTrait for GlShaderProgram {
	type Handle = crate::ShaderProgram;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::ShaderProgram { value }
	}
	#[inline]
	fn stuff(self) -> GlObjectType {
		GlObjectType::ShaderProgram(self)
	}
}


pub struct GlTexture2D {
	pub texture: GLuint,
	pub info: crate::Texture2DInfo,
}
impl GlObjectTrait for GlTexture2D {
	type Handle = crate::Texture2D;
	#[inline]
	fn create_handle(value: u32) -> Self::Handle {
		crate::Texture2D { value }
	}
	#[inline]
	fn stuff(self) -> GlObjectType {
		GlObjectType::Texture2D(self)
	}
}


pub struct GlObject {
	pub ref_count: u32,
	pub obj: GlObjectType,
}
impl GlObject {
	pub fn release(&self) {
		match &self.obj {
			GlObjectType::VertexBuffer(buf) => {
				gl_check!(gl::DeleteBuffers(1, &buf.buffer));
			}
			GlObjectType::IndexBuffer(buf) => {
				gl_check!(gl::DeleteBuffers(1, &buf.buffer));
			}
			GlObjectType::ShaderProgram(shader) => {
				shader::release(&shader);
			}
			GlObjectType::Texture2D(texture) => {
				texture2d::release(&texture);
			}
		}
	}
}

pub enum GlObjectType {
	VertexBuffer(GlVertexBuffer),
	IndexBuffer(GlIndexBuffer),
	ShaderProgram(GlShaderProgram),
	Texture2D(GlTexture2D),
}

pub struct ObjectMap {
	objects: HashMap<u32, GlObject>,
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
		let Some(obj) = self.objects.get(&handle.value) else {
			return None;
		};
		let obj_type = match obj.obj {
			GlObjectType::VertexBuffer(_) => crate::ObjectType::VertexBuffer,
			GlObjectType::IndexBuffer(_) => crate::ObjectType::IndexBuffer,
			GlObjectType::ShaderProgram(_) => crate::ObjectType::ShaderProgram,
			GlObjectType::Texture2D(_) => crate::ObjectType::Texture2D,
		};
		return Some(obj_type);
	}
	#[track_caller]
	pub fn add_ref(&mut self, handle: crate::BaseObject) {
		if handle == crate::BaseObject::INVALID {
			return;
		}
		let Some(obj) = self.objects.get_mut(&handle.value) else {
			panic!("Invalid object handle: {:?}", handle);
		};
		obj.ref_count += 1;
	}
	#[track_caller]
	pub fn release(&mut self, handle: crate::BaseObject) -> u32 {
		if handle == crate::BaseObject::INVALID {
			return 0;
		}
		let hash_map::Entry::Occupied(mut entry) = self.objects.entry(handle.value) else {
			panic!("Invalid object handle: {:?}", handle);
		};
		let obj = entry.get_mut();
		if obj.ref_count == 0 {
			panic!("Object released too many times: {:?}", handle);
		}
		obj.ref_count -= 1;
		let ref_count = obj.ref_count;
		if ref_count == 0 {
			obj.release();
			entry.remove();
		}
		ref_count
	}

	pub fn insert<T: GlObjectTrait>(&mut self, obj: T) -> T::Handle {
		loop {
			self.last = self.last.wrapping_add(1);
			if !self.objects.contains_key(&self.last) {
				break;
			}
		}
		let value = self.last;
		self.objects.insert(value, GlObject { ref_count: 1, obj: obj.stuff() });
		T::create_handle(value)
	}
}
impl ObjectMap {
	pub fn get_vertex_buffer(&self, handle: crate::VertexBuffer) -> Option<&GlVertexBuffer> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let GlObjectType::VertexBuffer(ref buf) = object.obj else { return None };
		return Some(buf);
	}
	pub fn get_index_buffer(&self, handle: crate::IndexBuffer) -> Option<&GlIndexBuffer> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let GlObjectType::IndexBuffer(ref buf) = object.obj else { return None };
		return Some(buf);
	}
	pub fn get_shader_program(&self, handle: crate::ShaderProgram) -> Option<&GlShaderProgram> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let GlObjectType::ShaderProgram(ref shader) = object.obj else { return None };
		return Some(shader);
	}
	pub fn get_texture2d(&self, handle: crate::Texture2D) -> Option<&GlTexture2D> {
		let Some(object) = self.objects.get(&handle.value) else { return None };
		let GlObjectType::Texture2D(ref texture) = object.obj else { return None };
		return Some(texture);
	}
	pub fn get_texture2d_mut(&mut self, handle: crate::Texture2D) -> Option<&mut GlTexture2D> {
		let Some(object) = self.objects.get_mut(&handle.value) else { return None };
		let GlObjectType::Texture2D(ref mut texture) = object.obj else { return None };
		return Some(texture);
	}
}
