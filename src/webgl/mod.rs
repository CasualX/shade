use std::mem;

mod api;

use crate::resources::{Resource, ResourceMap};
use crate::handle::Handle;

fn log(s: &str) {
	unsafe { api::consoleLog(s.as_ptr(), s.len()) };
}

struct GlBlend {
	sfactor: api::types::GLenum,
	dfactor: api::types::GLenum,
	equation: api::types::GLenum,
}
fn gl_blend(blend_mode: crate::BlendMode) {
	let p = match blend_mode {
		crate::BlendMode::Solid => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ZERO,
			equation: api::FUNC_ADD,
		},
		crate::BlendMode::Alpha => GlBlend {
			sfactor: api::SRC_ALPHA,
			dfactor: api::ONE_MINUS_SRC_ALPHA,
			equation: api::FUNC_ADD,
		},
		crate::BlendMode::Additive => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ONE,
			equation: api::FUNC_ADD,
		},
		crate::BlendMode::Lighten => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ONE,
			equation: api::MAX,
		},
		crate::BlendMode::Screen => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ONE_MINUS_SRC_COLOR,
			equation: api::FUNC_ADD,
		},
		crate::BlendMode::Darken => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ONE,
			equation: api::MIN,
		},
		crate::BlendMode::Multiply => GlBlend {
			sfactor: api::DST_COLOR,
			dfactor: api::ZERO,
			equation: api::FUNC_ADD,
		},
	};
	unsafe { api::enable(api::BLEND) };
	unsafe { api::blendFunc(p.sfactor, p.dfactor) };
	unsafe { api::blendEquation(p.equation) };
}

fn gl_depth_test(depth_test: Option<crate::DepthTest>) {
	if let Some(depth_test) = depth_test {
		let func = match depth_test {
			crate::DepthTest::Never => api::NEVER,
			crate::DepthTest::Less => api::LESS,
			crate::DepthTest::Equal => api::EQUAL,
			crate::DepthTest::LessEqual => api::LEQUAL,
			crate::DepthTest::Greater => api::GREATER,
			crate::DepthTest::NotEqual => api::NOTEQUAL,
			crate::DepthTest::GreaterEqual => api::GEQUAL,
			crate::DepthTest::Always => api::ALWAYS,
		};
		unsafe { api::enable(api::DEPTH_TEST) };
		unsafe { api::depthFunc(func) };
	}
	else {
		unsafe { api::disable(api::DEPTH_TEST) };
	}
}

fn gl_cull_face(cull_mode: Option<crate::CullMode>) {
	if let Some(cull_mode) = cull_mode {
		let mode = match cull_mode {
			crate::CullMode::CCW => api::FRONT,
			crate::CullMode::CW => api::BACK,
		};
		unsafe { api::enable(api::CULL_FACE) };
		unsafe { api::cullFace(mode) };
	}
	else {
		unsafe { api::disable(api::CULL_FACE) };
	}
}

fn gl_scissor(scissor: &Option<cvmath::Rect<i32>>) {
	if let Some(scissor) = scissor {
		unsafe { api::enable(api::SCISSOR_TEST) };
		unsafe { api::scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()) };
	}
	else {
		unsafe { api::disable(api::SCISSOR_TEST) };
	}
}

fn gl_viewport(viewport: &cvmath::Rect<i32>) {
	unsafe { api::viewport(viewport.mins.x, viewport.mins.y, viewport.width(), viewport.height()) };
}

fn gl_texture_id(textures: &ResourceMap<WebGLTexture2D>, id: crate::Texture2D) -> api::types::GLuint {
	if let Some(texture) = textures.get(id) {
		return texture.texture;
	}
	else {
		return 0;
	}
}

#[inline]
fn gl_mat_order(order: crate::MatrixLayout) -> api::types::GLboolean {
	match order {
		crate::MatrixLayout::ColumnMajor => api::FALSE,
		crate::MatrixLayout::RowMajor => api::TRUE,
	}
}

fn gl_attributes(shader: &WebGLShader, data: &[crate::DrawVertexBuffer], map: &ResourceMap<WebGLBuffer>) {
	for vb in data {
		let Some(buf) = map.get(vb.buffer) else { continue }; // Validated in draw calls
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };

		for attr in vb.layout.attributes {
			let location = unsafe { api::getAttribLocation(shader.program, attr.name.as_ptr(), attr.name.len()) };
			if location < 0 {
				continue; // Attribute not found in shader
			}
			let location = location as u32;
			unsafe { api::enableVertexAttribArray(location) };
			let size = attr.format.size() as api::types::GLint;
			let type_ = match attr.format.ty() {
				crate::VertexAttributeType::F32 => api::FLOAT,
				crate::VertexAttributeType::F64 => unimplemented!("F64 attributes are not supported in WebGL"),
				crate::VertexAttributeType::I32 => unimplemented!("I32 attributes are not supported in WebGL"),
				crate::VertexAttributeType::U32 => unimplemented!("U32 attributes are not supported in WebGL"),
				crate::VertexAttributeType::I16 => api::SHORT,
				crate::VertexAttributeType::U16 => api::UNSIGNED_SHORT,
				crate::VertexAttributeType::I8 => api::BYTE,
				crate::VertexAttributeType::U8 => api::UNSIGNED_BYTE,
			};
			let normalized = if attr.format.normalized() { api::TRUE } else { api::FALSE };
			if vb.layout.size >= 256 {
				panic!("Vertex attribute size too large: {}", vb.layout.size);
			}
			unsafe { api::vertexAttribPointer(location, size, type_, normalized, vb.layout.size as api::types::GLsizei, attr.offset as api::types::GLintptr) };
		}
	}
}

fn gl_uniforms(ub: &crate::UniformRef, shader: &WebGLShader, textures: &ResourceMap<WebGLTexture2D>) {
	unsafe { api::useProgram(shader.program) };

	for uattr in ub.layout.attributes {
		let data_ptr = unsafe { ub.data_ptr.offset(uattr.offset as isize) };
		if let Some(location) = shader.uniform_location(uattr.name) {
			// println!("Uniform: {} (index: {})", uattr.name, i);
			match uattr.ty {
				// crate::UniformType::D1 => unsafe { api::uniform1dv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::D2 => unsafe { api::uniform2dv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::D3 => unsafe { api::uniform3dv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::D4 => unsafe { api::uniform4dv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::F1 => unsafe { api::uniform1fv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::F2 => unsafe { api::uniform2fv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::F3 => unsafe { api::uniform3fv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::F4 => unsafe { api::uniform4fv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::I1 => unsafe { api::uniform1iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::I2 => unsafe { api::uniform2iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::I3 => unsafe { api::uniform3iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::I4 => unsafe { api::uniform4iv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::U1 => unsafe { api::uniform1uiv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::U2 => unsafe { api::uniform2uiv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::U3 => unsafe { api::uniform3uiv(location, uattr.len as i32, data_ptr as *const _) },
				// crate::UniformType::U4 => unsafe { api::uniform4uiv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::B1 => unsafe { api::uniform1iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::B2 => unsafe { api::uniform2iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::B3 => unsafe { api::uniform3iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::B4 => unsafe { api::uniform4iv(location, uattr.len as i32, data_ptr as *const _) },
				crate::UniformType::Mat2x2 { order } => unsafe { api::uniformMatrix2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat2x3 { order } => unsafe { api::uniformMatrix2x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat2x4 { order } => unsafe { api::uniformMatrix2x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat3x2 { order } => unsafe { api::uniformMatrix3x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat3x3 { order } => unsafe { api::uniformMatrix3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat3x4 { order } => unsafe { api::uniformMatrix3x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat4x2 { order } => unsafe { api::uniformMatrix4x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat4x3 { order } => unsafe { api::uniformMatrix4x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Mat4x4 { order } => unsafe { api::uniformMatrix4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) },
				crate::UniformType::Sampler2D(index) => {
					let id = unsafe { *(data_ptr as *const crate::Texture2D) };
					let texture = gl_texture_id(textures, id);
					unsafe { api::activeTexture(api::TEXTURE0 + index as u32) };
					unsafe { api::bindTexture(api::TEXTURE_2D, texture) };
				}
				_ => unimplemented!("Uniform type not implemented: {:?}", uattr.ty),
			}
		}
		else {
			// panic!("Uniform not found: {}", uattr.name);
		}
	}
}

struct WebGLBuffer {
	buffer: api::types::GLuint,
	_size: usize,
	usage: crate::BufferUsage,
}
impl Resource for WebGLBuffer {
	type Handle = crate::Buffer;
}

struct WebGLShaderActiveUniform {
	location: api::types::GLint,
	namelen: u8,
	namebuf: [u8; 64],
	_size: api::types::GLint,
	_ty: api::types::GLenum,
}
impl WebGLShaderActiveUniform {
	fn name(&self) -> &str {
		std::str::from_utf8(&self.namebuf[..self.namelen as usize]).unwrap_or("err")
	}
}

struct WebGLShader {
	program: api::types::GLuint,
	compile_log: String,

	active_uniforms: Vec<WebGLShaderActiveUniform>,
}
impl WebGLShader {
	fn uniform_location(&self, name: &str) -> Option<api::types::GLint> {
		for au in &self.active_uniforms {
			if au.name() == name {
				return Some(au.location);
			}
		}
		return None;
	}
}
impl Resource for WebGLShader {
	type Handle = crate::Shader;
}

struct WebGLTexture2D {
	texture: api::types::GLuint,
	info: crate::Texture2DInfo,
}
impl Resource for WebGLTexture2D {
	type Handle = crate::Texture2D;
}

#[allow(dead_code)]
struct WebGLSurface {
	texture: crate::Texture2D,
	frame_buf: api::types::GLuint,
	depth_buf: api::types::GLuint,
	tex_buf: api::types::GLuint,
	format: crate::SurfaceFormat,
	width: i32,
	height: i32,
}

impl Resource for WebGLSurface {
	type Handle = crate::Surface;
}

pub struct WebGLGraphics {
	buffers: ResourceMap<WebGLBuffer>,
	shaders: ResourceMap<WebGLShader>,
	textures2d: ResourceMap<WebGLTexture2D>,
	surfaces: ResourceMap<WebGLSurface>,
	drawing: bool,
}

impl WebGLGraphics {
	pub fn new() -> Self {
		WebGLGraphics {
			buffers: ResourceMap::new(),
			shaders: ResourceMap::new(),
			textures2d: ResourceMap::new(),
			surfaces: ResourceMap::new(),
			drawing: false,
		}
	}
}

impl crate::IGraphics for WebGLGraphics {
	fn begin(&mut self) -> Result<(), crate::GfxError> {
		if self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		self.drawing = true;
		Ok(())
	}

	fn clear(&mut self, args: &crate::ClearArgs) -> Result<(), crate::GfxError> {
		if !self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		if let Some(scissor) = args.scissor {
			unsafe { api::enable(api::SCISSOR_TEST) };
			unsafe { api::scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()) };
		}
		else {
			unsafe { api::disable(api::SCISSOR_TEST) };
		}

		let mut mask = 0;
		if let Some(color) = args.color {
			unsafe { api::clearColor(color.x, color.y, color.z, color.w) };
			mask |= api::COLOR_BUFFER_BIT;
		}
		if let Some(depth) = args.depth {
			unsafe { api::clearDepth(depth as f64) };
			mask |= api::DEPTH_BUFFER_BIT;
		}
		if let Some(stencil) = args.stencil{
			unsafe { api::clearStencil(stencil as i32) };
			mask |= api::STENCIL_BUFFER_BIT;
		}
		unsafe { api::clear(mask) };

		Ok(())
	}

	fn draw(&mut self, args: &crate::DrawArgs) -> Result<(), crate::GfxError> {
		if !self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		for data in args.vertices {
			let Some(_) = self.buffers.get(data.buffer) else { return Err(crate::GfxError::InvalidHandle) };
		}
		let Some(shader) = self.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidHandle) };

		if args.vertex_end < args.vertex_start {
			return Err(crate::GfxError::IndexOutOfBounds);
		}
		if args.vertex_start == args.vertex_end {
			return Ok(());
		}

		gl_blend(args.blend_mode);
		gl_depth_test(args.depth_test);
		gl_cull_face(args.cull_mode);
		gl_scissor(&args.scissor);
		gl_viewport(&args.viewport);

		gl_attributes(shader, args.vertices, &self.buffers);
		gl_uniforms(&args.uniforms, shader, &self.textures2d);

		let mode = match args.prim_type {
			crate::PrimType::Lines => api::LINES,
			crate::PrimType::Triangles => api::TRIANGLES,
		};
		if args.instances >= 0 {
			unsafe { api::drawArraysInstanced(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32, args.instances) };
		}
		else {
			unsafe { api::drawArrays(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32) };
		}

		Ok(())
	}

	fn draw_indexed(&mut self, args: &crate::DrawIndexedArgs) -> Result<(), crate::GfxError> {
		if !self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		for data in args.vertices {
			let Some(_) = self.buffers.get(data.buffer) else { return Err(crate::GfxError::InvalidHandle) };
		}
		let Some(ib) = self.buffers.get(args.indices) else { return Err(crate::GfxError::InvalidHandle) };
		let Some(shader) = self.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidHandle) };

		if args.index_end < args.index_start || args.vertex_end < args.vertex_start {
			return Err(crate::GfxError::IndexOutOfBounds);
		}
		if args.index_start == args.index_end {
			return Ok(());
		}

		gl_blend(args.blend_mode);
		gl_depth_test(args.depth_test);
		gl_cull_face(args.cull_mode);
		gl_scissor(&args.scissor);
		gl_viewport(&args.viewport);

		gl_attributes(shader, args.vertices, &self.buffers);
		unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, ib.buffer) };
		gl_uniforms(&args.uniforms, shader, &self.textures2d);

		let mode = match args.prim_type {
			crate::PrimType::Lines => api::LINES,
			crate::PrimType::Triangles => api::TRIANGLES,
		};
		let count = args.index_end - args.index_start;
		let type_ = match args.index_type {
			crate::IndexType::U8 => gl::UNSIGNED_BYTE,
			crate::IndexType::U16 => gl::UNSIGNED_SHORT,
			crate::IndexType::U32 => gl::UNSIGNED_INT,
		};
		let offset = args.index_start * args.index_type.size() as u32;
		if args.instances >= 0 {
			unsafe { api::drawElementsInstanced(mode, count as i32, type_, offset as api::types::GLintptr, args.instances) };
		}
		else {
			unsafe { api::drawElements(mode, count as i32, type_, offset as api::types::GLintptr) };
		}

		Ok(())
	}

	fn end(&mut self) -> Result<(), crate::GfxError> {
		self.drawing = false;
		Ok(())
	}

	fn buffer_create(&mut self, name: Option<&str>, _size: usize, usage: crate::BufferUsage) -> Result<crate::Buffer, crate::GfxError> {
		let buffer = unsafe { api::createBuffer() };

		let id = self.buffers.insert(name, WebGLBuffer { buffer, _size, usage });
		return Ok(id);
	}

	fn buffer_find(&mut self, name: &str) -> Result<crate::Buffer, crate::GfxError> {
		self.buffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn buffer_set_data(&mut self, id: crate::Buffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.buffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as api::types::GLsizeiptr;
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

	fn buffer_delete(&mut self, id: crate::Buffer, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(vb) = self.buffers.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		unsafe { api::deleteBuffer(vb.buffer) };
		Ok(())
	}

	fn shader_create(&mut self, name: Option<&str>) -> Result<crate::Shader, crate::GfxError> {
		let program = unsafe { api::createProgram() };
		let id = self.shaders.insert(name, WebGLShader { program, compile_log: String::new(), active_uniforms: Vec::new() });
		return Ok(id);
	}

	fn shader_find(&mut self, name: &str) -> Result<crate::Shader, crate::GfxError> {
		self.shaders.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn shader_compile(&mut self, id: crate::Shader, vertex_source: &str, fragment_source: &str) -> Result<(), crate::GfxError> {
		let Some(shader) = self.shaders.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let mut success = true;

		shader.active_uniforms.clear();

		let vertex_shader = unsafe { api::createShader(api::VERTEX_SHADER) };
		unsafe { api::shaderSource(vertex_shader, vertex_source.as_ptr(), vertex_source.len()) };
		unsafe { api::compileShader(vertex_shader) };
		let status = unsafe { api::getShaderParameter(vertex_shader, api::COMPILE_STATUS) };
		if status == 0 {
			unsafe { api::getShaderInfoLog(vertex_shader) };
			success = false;
		}

		let fragment_shader = unsafe { api::createShader(api::FRAGMENT_SHADER) };
		unsafe { api::shaderSource(fragment_shader, fragment_source.as_ptr(), fragment_source.len()) };
		unsafe { api::compileShader(fragment_shader) };
		let status = unsafe { api::getShaderParameter(fragment_shader, api::COMPILE_STATUS) };
		if status == 0 {
			unsafe { api::getShaderInfoLog(fragment_shader) };
			success = false;
		}

		if success {
			unsafe { api::attachShader(shader.program, vertex_shader) };
			unsafe { api::attachShader(shader.program, fragment_shader) };
			unsafe { api::linkProgram(shader.program) };
			let status = unsafe { api::getProgramParameter(shader.program, api::LINK_STATUS) };
			if status == 0 {
				unsafe { api::getProgramInfoLog(shader.program) };
				success = false;
			}
			else {
				unsafe { api::useProgram(shader.program) };
				let count = unsafe { api::getProgramParameter(shader.program, gl::ACTIVE_UNIFORMS) };
				for i in 0..count {
					let mut name_len = 0;
					let mut size = 0;
					let mut ty = 0;
					let mut name = [0; 64];
					unsafe { api::getActiveUniform(shader.program, i as u32, 64, &mut name_len, &mut size, &mut ty, name.as_mut_ptr()) };
					shader.active_uniforms.push(WebGLShaderActiveUniform {
						location: i,
						namelen: name_len as u8,
						namebuf: name,
						_size: size,
						_ty: ty,
					});
				}
			}
		}

		unsafe { api::deleteShader(vertex_shader) };
		unsafe { api::deleteShader(fragment_shader) };
		return if success { Ok(()) } else { Err(crate::GfxError::ShaderCompileError) };
	}

	fn shader_compile_log(&mut self, id: crate::Shader) -> Result<String, crate::GfxError> {
		todo!()
	}

	fn shader_delete(&mut self, id: crate::Shader, free_handle: bool) -> Result<(), crate::GfxError> {
		todo!()
	}

	fn texture2d_create(&mut self, name: Option<&str>, info: &crate::Texture2DInfo) -> Result<crate::Texture2D, crate::GfxError> {
		todo!()
	}

	fn texture2d_find(&mut self, name: &str) -> Result<crate::Texture2D, crate::GfxError> {
		todo!()
	}

	fn texture2d_set_data(&mut self, id: crate::Texture2D, data: &[u8]) -> Result<(), crate::GfxError> {
		todo!()
	}

	fn texture2d_get_info(&mut self, id: crate::Texture2D) -> Result<crate::Texture2DInfo, crate::GfxError> {
		todo!()
	}

	fn texture2d_delete(&mut self, id: crate::Texture2D, free_handle: bool) -> Result<(), crate::GfxError> {
		todo!()
	}

	fn texture2darray_create(&mut self, name: Option<&str>, info: &crate::Texture2DArrayInfo) -> Result<crate::Texture2DArray, crate::GfxError> {
		todo!()
	}

	fn texture2darray_find(&mut self, name: &str) -> Result<crate::Texture2DArray, crate::GfxError> {
		todo!()
	}

	fn texture2darray_set_data(&mut self, id: crate::Texture2DArray, index: usize, data: &[u8]) -> Result<(), crate::GfxError> {
		todo!()
	}

	fn texture2darray_get_info(&mut self, id: crate::Texture2DArray) -> Result<crate::Texture2DArrayInfo, crate::GfxError> {
		todo!()
	}

	fn texture2darray_delete(&mut self, id: crate::Texture2DArray, free_handle: bool) -> Result<(), crate::GfxError> {
		todo!()
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

	fn surface_delete(&mut self, id: crate::Surface, free_handle: bool) -> Result<(), crate::GfxError> {
		todo!()
	}

}
