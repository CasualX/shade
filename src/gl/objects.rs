use super::*;

pub struct GlVertexBuffer {
	pub generation: u32,
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub layout: &'static crate::VertexLayout,
}
impl crate::Resource for GlVertexBuffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GlVertexBuffer({:p})", self)
	}
}
impl crate::VertexBuffer for GlVertexBuffer {
	fn layout(&self) -> &'static crate::VertexLayout { self.layout }
}
impl Drop for GlVertexBuffer {
	fn drop(&mut self) {
		if generation::is_current(self.generation) {
			gl_check!(gl::DeleteBuffers(1, &self.buffer));
		}
	}
}

pub struct GlIndexBuffer {
	pub generation: u32,
	pub buffer: GLuint,
	pub size: usize,
	pub _usage: crate::BufferUsage,
	pub ty: crate::IndexType,
}
impl crate::Resource for GlIndexBuffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GlIndexBuffer({:p})", self)
	}
}
impl crate::IndexBuffer for GlIndexBuffer {
	fn index_type(&self) -> crate::IndexType { self.ty }
}
impl Drop for GlIndexBuffer {
	fn drop(&mut self) {
		if generation::is_current(self.generation) {
			gl_check!(gl::DeleteBuffers(1, &self.buffer));
		}
	}
}

#[allow(dead_code)]
pub struct GlActiveAttrib {
	pub location: GLuint,
	pub size: GLint,
	pub ty: GLenum,
}
pub struct GlActiveUniform {
	pub location: GLint,
	pub array_size: GLint,
	pub ty: GLenum,
	pub texture_unit: i8,
}
pub struct GlShaderProgram {
	pub generation: u32,
	pub program: GLuint,
	pub attribs: HashMap<NameBuf, GlActiveAttrib>,
	pub uniforms: HashMap<NameBuf, GlActiveUniform>,
}
impl crate::Resource for GlShaderProgram {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GlShaderProgram({:p})", self)
	}
}
impl crate::ShaderProgram for GlShaderProgram {
}
impl Drop for GlShaderProgram {
	fn drop(&mut self) {
		if generation::is_current(self.generation) {
			gl_check!(gl::DeleteProgram(self.program));
		}
	}
}

pub struct GlTexture2D {
	pub generation: u32,
	pub texture: GLuint,
	pub info: crate::Texture2DInfo,
}
impl crate::Resource for GlTexture2D {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GlTexture2D({:p})", self)
	}
}
impl crate::Texture2D for GlTexture2D {
	fn info(&self) -> &crate::Texture2DInfo { &self.info }
}
impl Drop for GlTexture2D {
	fn drop(&mut self) {
		if generation::is_current(self.generation) {
			gl_check!(gl::DeleteTextures(1, &self.texture));
		}
	}
}

pub struct GlTextureRef<'a> {
	pub texture: GLuint,
	pub info: &'a crate::Texture2DInfo,
}

#[track_caller]
pub fn vertex_buffer(buffer: &dyn crate::VertexBuffer) -> &GlVertexBuffer {
	let buffer: &dyn std::any::Any = buffer;
	buffer.downcast_ref::<GlVertexBuffer>().expect("OpenGL backend received a foreign VertexBuffer")
}

#[track_caller]
pub fn vertex_buffer_mut(buffer: &mut dyn crate::VertexBuffer) -> &mut GlVertexBuffer {
	let buffer: &mut dyn std::any::Any = buffer;
	buffer.downcast_mut::<GlVertexBuffer>().expect("OpenGL backend received a foreign VertexBuffer")
}

#[track_caller]
pub fn index_buffer(buffer: &dyn crate::IndexBuffer) -> &GlIndexBuffer {
	let buffer: &dyn std::any::Any = buffer;
	buffer.downcast_ref::<GlIndexBuffer>().expect("OpenGL backend received a foreign IndexBuffer")
}

#[track_caller]
pub fn index_buffer_mut(buffer: &mut dyn crate::IndexBuffer) -> &mut GlIndexBuffer {
	let buffer: &mut dyn std::any::Any = buffer;
	buffer.downcast_mut::<GlIndexBuffer>().expect("OpenGL backend received a foreign IndexBuffer")
}

#[track_caller]
pub fn shader_program(shader: &dyn crate::ShaderProgram) -> &GlShaderProgram {
	let shader: &dyn std::any::Any = shader;
	shader.downcast_ref::<GlShaderProgram>().expect("OpenGL backend received a foreign ShaderProgram")
}

#[track_caller]
pub fn texture2d<'a>(this: &'a GlGraphics, texture: &'a dyn crate::Texture2D) -> GlTextureRef<'a> {
	let texture_any: &dyn std::any::Any = texture;
	if let Some(texture) = texture_any.downcast_ref::<GlTexture2D>() {
		return GlTextureRef { texture: texture.texture, info: &texture.info };
	}

	if !texture_any.is::<crate::DefaultTexture2D>() {
		panic!("OpenGL backend received a foreign Texture2D");
	}

	GlTextureRef {
		texture: this.texture2d_default,
		info: &crate::DefaultTexture2D::INFO,
	}
}

#[track_caller]
pub fn texture2d_mut(texture: &mut dyn crate::Texture2D) -> &mut GlTexture2D {
	let texture: &mut dyn std::any::Any = texture;
	assert!(!texture.is::<crate::DefaultTexture2D>(), "OpenGL backend cannot mutate the shared DefaultTexture2D");
	texture.downcast_mut::<GlTexture2D>().expect("OpenGL backend received a foreign Texture2D")
}
