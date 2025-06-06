/*!
OpenGL graphics backend.
*/

use std::{mem, ops};

/// Re-exported OpenGL bindings.
pub use gl as capi;

pub const MTSDF_FS: &str = include_str!("shaders/mtsdf.fs.glsl");
pub const MTSDF_VS: &str = include_str!("shaders/mtsdf.vs.glsl");

use crate::resources::{Resource, ResourceMap};
use crate::handle::Handle;

#[cfg(debug_assertions)]
macro_rules! gl_check {
	($e:expr) => {
		check(|| unsafe { $e })
	};
}
#[cfg(not(debug_assertions))]
macro_rules! gl_check {
	($e:expr) => {
		unsafe { $e }
	};
}

struct GlBuffer {
	buffer: gl::types::GLuint,
	_size: usize,
	usage: crate::BufferUsage,
}
impl Resource for GlBuffer {
	type Handle = crate::Buffer;
}

struct GlShaderActiveUniform {
	location: gl::types::GLint,
	namelen: u8,
	namebuf: [u8; 64],
	_size: gl::types::GLint,
	_ty: gl::types::GLenum,
}
impl GlShaderActiveUniform {
	fn name(&self) -> &str {
		std::str::from_utf8(&self.namebuf[..self.namelen as usize]).unwrap_or("err")
	}
}

struct GlShader {
	program: gl::types::GLuint,
	compile_log: String,

	active_uniforms: Vec<GlShaderActiveUniform>,
}
impl Resource for GlShader {
	type Handle = crate::Shader;
}
impl GlShader {
	fn uniform_location(&self, name: &str) -> Option<gl::types::GLint> {
		for au in &self.active_uniforms {
			if au.name() == name {
				return Some(au.location);
			}
		}
		return None;
	}
}

struct GlTexture2D {
	texture: gl::types::GLuint,
	info: crate::Texture2DInfo,
}
impl Resource for GlTexture2D {
	type Handle = crate::Texture2D;
}

struct GlTexture2DArray {
	texture: gl::types::GLuint,
	info: crate::Texture2DArrayInfo,
}
impl Resource for GlTexture2DArray {
	type Handle = crate::Texture2DArray;
}

#[allow(dead_code)]
struct GlSurface {
	texture: crate::Texture2D,
	frame_buf: gl::types::GLuint,
	depth_buf: gl::types::GLuint,
	tex_buf: gl::types::GLuint,
	format: crate::SurfaceFormat,
	width: i32,
	height: i32,
}
impl Resource for GlSurface {
	type Handle = crate::Surface;
}

struct GlBlend {
	sfactor: gl::types::GLenum,
	dfactor: gl::types::GLenum,
	equation: gl::types::GLenum,
}
fn gl_blend(blend_mode: crate::BlendMode) {
	let p = match blend_mode {
		crate::BlendMode::Solid => GlBlend {
			sfactor: gl::ONE,
			dfactor: gl::ZERO,
			equation: gl::FUNC_ADD,
		},
		crate::BlendMode::Alpha => GlBlend {
			sfactor: gl::SRC_ALPHA,
			dfactor: gl::ONE_MINUS_SRC_ALPHA,
			equation: gl::FUNC_ADD,
		},
		crate::BlendMode::Additive => GlBlend {
			sfactor: gl::ONE,
			dfactor: gl::ONE,
			equation: gl::FUNC_ADD,
		},
		crate::BlendMode::Lighten => GlBlend {
			sfactor: gl::ONE,
			dfactor: gl::ONE,
			equation: gl::MAX,
		},
		crate::BlendMode::Screen => GlBlend {
			sfactor: gl::ONE,
			dfactor: gl::ONE_MINUS_SRC_COLOR,
			equation: gl::FUNC_ADD,
		},
		crate::BlendMode::Darken => GlBlend {
			sfactor: gl::ONE,
			dfactor: gl::ONE,
			equation: gl::MIN,
		},
		crate::BlendMode::Multiply => GlBlend {
			sfactor: gl::DST_COLOR,
			dfactor: gl::ZERO,
			equation: gl::FUNC_ADD,
		},
	};
	gl_check!(gl::Enable(gl::BLEND));
	gl_check!(gl::BlendFunc(p.sfactor, p.dfactor));
	gl_check!(gl::BlendEquation(p.equation));
}

fn gl_scissor(scissor: &Option<cvmath::Rect<i32>>) {
	if let Some(scissor) = scissor {
		gl_check!(gl::Enable(gl::SCISSOR_TEST));
		gl_check!(gl::Scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()));
	}
	else {
		gl_check!(gl::Disable(gl::SCISSOR_TEST));
	}
}

fn gl_texture_id(textures: &ResourceMap<GlTexture2D>, id: crate::Texture2D) -> gl::types::GLuint {
	if let Some(texture) = textures.get(id) {
		return texture.texture;
	}
	else {
		return 0;
	}
}

fn gl_depth_test(depth_test: Option<crate::DepthTest>) {
	if let Some(depth_test) = depth_test {
		let func = match depth_test {
			crate::DepthTest::Never => gl::NEVER,
			crate::DepthTest::Less => gl::LESS,
			crate::DepthTest::Equal => gl::EQUAL,
			crate::DepthTest::LessEqual => gl::LEQUAL,
			crate::DepthTest::Greater => gl::GREATER,
			crate::DepthTest::NotEqual => gl::NOTEQUAL,
			crate::DepthTest::GreaterEqual => gl::GEQUAL,
			crate::DepthTest::Always => gl::ALWAYS,
		};
		gl_check!(gl::Enable(gl::DEPTH_TEST));
		gl_check!(gl::DepthFunc(func));
	}
	else {
		gl_check!(gl::Disable(gl::DEPTH_TEST));
	}
}

fn gl_cull_face(cull_mode: Option<crate::CullMode>) {
	if let Some(cull_mode) = cull_mode {
		let mode = match cull_mode {
			crate::CullMode::CCW => gl::FRONT,
			crate::CullMode::CW => gl::BACK,
		};
		gl_check!(gl::Enable(gl::CULL_FACE));
		gl_check!(gl::CullFace(mode));
	}
	else {
		gl_check!(gl::Disable(gl::CULL_FACE));
	}
}

#[inline]
fn gl_mat_order(order: crate::MatrixLayout) -> gl::types::GLboolean {
	match order {
		crate::MatrixLayout::ColumnMajor => gl::FALSE,
		crate::MatrixLayout::RowMajor => gl::TRUE,
	}
}

fn gl_uniforms(ub: &crate::UniformRef, shader: &GlShader, textures: &ResourceMap<GlTexture2D>) {
	gl_check!(gl::UseProgram(shader.program));

	for uattr in ub.layout.attributes {
		let data_ptr = unsafe { ub.data_ptr.offset(uattr.offset as isize) };
		if let Some(location) = shader.uniform_location(uattr.name) {
			// println!("Uniform: {} (index: {})", uattr.name, i);
			match uattr.ty {
				crate::UniformType::D1 => gl_check!(gl::Uniform1dv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::D2 => gl_check!(gl::Uniform2dv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::D3 => gl_check!(gl::Uniform3dv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::D4 => gl_check!(gl::Uniform4dv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::F1 => gl_check!(gl::Uniform1fv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::F2 => gl_check!(gl::Uniform2fv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::F3 => gl_check!(gl::Uniform3fv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::F4 => gl_check!(gl::Uniform4fv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::I1 => gl_check!(gl::Uniform1iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::I2 => gl_check!(gl::Uniform2iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::I3 => gl_check!(gl::Uniform3iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::I4 => gl_check!(gl::Uniform4iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::U1 => gl_check!(gl::Uniform1uiv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::U2 => gl_check!(gl::Uniform2uiv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::U3 => gl_check!(gl::Uniform3uiv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::U4 => gl_check!(gl::Uniform4uiv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::B1 => gl_check!(gl::Uniform1iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::B2 => gl_check!(gl::Uniform2iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::B3 => gl_check!(gl::Uniform3iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::B4 => gl_check!(gl::Uniform4iv(location, uattr.len as i32, data_ptr as *const _)),
				crate::UniformType::Mat2x2 { order } => gl_check!(gl::UniformMatrix2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat2x3 { order } => gl_check!(gl::UniformMatrix2x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat2x4 { order } => gl_check!(gl::UniformMatrix2x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat3x2 { order } => gl_check!(gl::UniformMatrix3x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat3x3 { order } => gl_check!(gl::UniformMatrix3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat3x4 { order } => gl_check!(gl::UniformMatrix3x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat4x2 { order } => gl_check!(gl::UniformMatrix4x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat4x3 { order } => gl_check!(gl::UniformMatrix4x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Mat4x4 { order } => gl_check!(gl::UniformMatrix4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _)),
				crate::UniformType::Sampler2D(index) => {
					let id = unsafe { *(data_ptr as *const crate::Texture2D) };
					let texture = gl_texture_id(textures, id);
					gl_check!(gl::ActiveTexture(gl::TEXTURE0 + index as u32));
					gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture));
				}
			}
		}
		else {
			// panic!("Uniform not found: {}", uattr.name);
		}
	}
}

fn gl_attributes(dynamic_vao: gl::types::GLuint, shader: &GlShader, data: &[crate::DrawVertexBuffer], map: &ResourceMap<GlBuffer>) {
	gl_check!(gl::BindVertexArray(dynamic_vao));

	for dvb in data {
		let Some(buf) = map.get(dvb.buffer) else { continue }; // Validated in draw calls
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));

		let layout = dvb.layout;
		for attr in layout.attributes {
			let mut namebuf = [0u8; 64];
			namebuf[..attr.name.len()].copy_from_slice(attr.name.as_bytes());
			namebuf[attr.name.len()] = 0; // Null-terminate the string
			let location: i32 = gl_check!(gl::GetAttribLocation(shader.program, namebuf.as_ptr() as *const _));
			if location < 0 {
				continue; // Attribute not found
			}
			let location = location as u32;
			let size = attr.format.size() as gl::types::GLint;
			let normalized = if attr.format.normalized() { gl::TRUE } else { gl::FALSE };
			let type_ = match attr.format.ty() {
				crate::VertexAttributeType::F32 => gl::FLOAT,
				crate::VertexAttributeType::F64 => gl::FLOAT,
				crate::VertexAttributeType::I32 => gl::INT,
				crate::VertexAttributeType::U32 => gl::UNSIGNED_INT,
				crate::VertexAttributeType::I16 => gl::SHORT,
				crate::VertexAttributeType::U16 => gl::UNSIGNED_SHORT,
				crate::VertexAttributeType::I8 => gl::BYTE,
				crate::VertexAttributeType::U8 => gl::UNSIGNED_BYTE,
			};
			gl_check!(gl::VertexAttribPointer(location, size, type_, normalized, layout.size as i32, attr.offset as usize as *const _));
			let divisor = match dvb.divisor {
				crate::VertexDivisor::PerVertex => 0,
				crate::VertexDivisor::PerInstance => 1,
			};
			gl_check!(gl::VertexAttribDivisor(location, divisor));
			gl_check!(gl::EnableVertexAttribArray(location));
		}
	}
}

fn gl_texture_wrap(wrap: crate::TextureWrap) -> gl::types::GLenum {
	match wrap {
		crate::TextureWrap::ClampEdge => gl::CLAMP_TO_EDGE,
		crate::TextureWrap::ClampBorder => gl::CLAMP_TO_BORDER,
		crate::TextureWrap::Repeat => gl::REPEAT,
		crate::TextureWrap::Mirror => gl::MIRRORED_REPEAT,
	}
}
fn gl_texture_filter(filter: crate::TextureFilter) -> gl::types::GLenum {
	match filter {
		crate::TextureFilter::Nearest => gl::NEAREST,
		crate::TextureFilter::Linear => gl::LINEAR,
	}
}

pub struct GlGraphics {
	buffers: ResourceMap<GlBuffer>,
	shaders: ResourceMap<GlShader>,
	textures2d: ResourceMap<GlTexture2D>,
	textures2darray: ResourceMap<GlTexture2DArray>,
	surfaces: ResourceMap<GlSurface>,
	dynamic_vao: gl::types::GLuint,
	drawing: bool,
}

impl GlGraphics {
	pub fn new() -> Self {
		let mut dynamic_vao = 0;
		gl_check!(gl::GenVertexArrays(1, &mut dynamic_vao));
		GlGraphics {
			buffers: ResourceMap::new(),
			shaders: ResourceMap::new(),
			textures2d: ResourceMap::new(),
			textures2darray: ResourceMap::new(),
			surfaces: ResourceMap::new(),
			dynamic_vao,
			drawing: false,
		}
	}
}

impl crate::IGraphics for GlGraphics {
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
			gl_check!(gl::Enable(gl::SCISSOR_TEST));
			gl_check!(gl::Scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()));
		}
		else {
			gl_check!(gl::Disable(gl::SCISSOR_TEST));
		}

		let mut mask = 0;
		if let Some(color) = args.color {
			gl_check!(gl::ClearColor(color.x, color.y, color.z, color.w));
			mask |= gl::COLOR_BUFFER_BIT;
		}
		if let Some(depth) = args.depth {
			gl_check!(gl::ClearDepth(depth as f64));
			mask |= gl::DEPTH_BUFFER_BIT;
		}
		if let Some(stencil) = args.stencil{
			gl_check!(gl::ClearStencil(stencil as i32));
			mask |= gl::STENCIL_BUFFER_BIT;
		}
		gl_check!(gl::Clear(mask));

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
		gl_check!(gl::Viewport(args.viewport.mins.x, args.viewport.mins.y, args.viewport.width(), args.viewport.height()));

		gl_check!(gl::UseProgram(shader.program));

		gl_attributes(self.dynamic_vao, shader, args.vertices, &self.buffers);
		gl_uniforms(&args.uniforms, shader, &self.textures2d);

		let mode = match args.prim_type {
			crate::PrimType::Lines => gl::LINES,
			crate::PrimType::Triangles => gl::TRIANGLES,
		};
		if args.instances >= 0 {
			gl_check!(gl::DrawArraysInstanced(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32, args.instances));
		}
		else {
			gl_check!(gl::DrawArrays(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32));
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
		gl_check!(gl::Viewport(args.viewport.mins.x, args.viewport.mins.y, args.viewport.width(), args.viewport.height()));

		gl_check!(gl::UseProgram(shader.program));

		gl_attributes(self.dynamic_vao, shader, args.vertices, &self.buffers);
		gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.buffer));

		gl_uniforms(&args.uniforms, shader, &self.textures2d);

		let mode = match args.prim_type {
			crate::PrimType::Lines => gl::LINES,
			crate::PrimType::Triangles => gl::TRIANGLES,
		};
		let count = args.index_end - args.index_start;
		let type_ = match args.index_type {
			crate::IndexType::U8 => gl::UNSIGNED_BYTE,
			crate::IndexType::U16 => gl::UNSIGNED_SHORT,
			crate::IndexType::U32 => gl::UNSIGNED_INT,
		};
		let offset = args.index_start * args.index_type.size() as u32;
		if args.instances >= 0 {
			gl_check!(gl::DrawElementsInstanced(mode, count as i32, type_, offset as *const _, args.instances));
		}
		else {
			gl_check!(gl::DrawElements(mode, count as i32, type_, offset as *const _));
		}

		Ok(())
	}

	fn end(&mut self) -> Result<(), crate::GfxError> {
		self.drawing = false;
		Ok(())
	}

	fn buffer_create(&mut self, name: Option<&str>, size: usize, usage: crate::BufferUsage) -> Result<crate::Buffer, crate::GfxError> {
		let mut buffer = 0;
		gl_check!(gl::GenBuffers(1, &mut buffer));

		let id = self.buffers.insert(name, GlBuffer { buffer, _size: size, usage });
		return Ok(id);
	}

	fn buffer_find(&mut self, name: &str) -> Result<crate::Buffer, crate::GfxError> {
		self.buffers.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn buffer_set_data(&mut self, id: crate::Buffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(buf) = self.buffers.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let size = mem::size_of_val(data) as gl::types::GLsizeiptr;
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

	fn buffer_delete(&mut self, id: crate::Buffer, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(buf) = self.buffers.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteBuffers(1, &buf.buffer));
		Ok(())
	}

	fn shader_create(&mut self, name: Option<&str>) -> Result<crate::Shader, crate::GfxError> {
		let program = gl_check!(gl::CreateProgram());
		let id = self.shaders.insert(name, GlShader { program, compile_log: String::new(), active_uniforms: Vec::new() });
		return Ok(id);
	}

	fn shader_find(&mut self, name: &str) -> Result<crate::Shader, crate::GfxError> {
		self.shaders.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn shader_compile(&mut self, id: crate::Shader, vertex_source: &str, fragment_source: &str) -> Result<(), crate::GfxError> {
		let Some(shader) = self.shaders.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
		let mut success = true;
		let mut status = 0;

		shader.active_uniforms.clear();

		let vertex_shader = gl_check!(gl::CreateShader(gl::VERTEX_SHADER));
		gl_check!(gl::ShaderSource(vertex_shader, 1, &(vertex_source.as_ptr() as *const _), &(vertex_source.len() as gl::types::GLint)));
		gl_check!(gl::CompileShader(vertex_shader));
		gl_check!(gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut status));
		if status == 0 {
			let mut log_len = 0;
			gl_check!(gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_len));
			let mut log = vec![0; log_len as usize];
			gl_check!(gl::GetShaderInfoLog(vertex_shader, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _));
			shader.compile_log.push_str("# Vertex shader compile log:\n");
			shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
			success = false;
		}

		let fragment_shader = gl_check!(gl::CreateShader(gl::FRAGMENT_SHADER));
		gl_check!(gl::ShaderSource(fragment_shader, 1, &(fragment_source.as_ptr() as *const _), &(fragment_source.len() as gl::types::GLint)));
		gl_check!(gl::CompileShader(fragment_shader));
		gl_check!(gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut status));
		if status == 0 {
			let mut log_len = 0;
			gl_check!(gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_len));
			let mut log = vec![0; log_len as usize];
			gl_check!(gl::GetShaderInfoLog(fragment_shader, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _));
			shader.compile_log.push_str("# Fragment shader compile log:\n");
			shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
			success = false;
		}

		if success {
			gl_check!(gl::AttachShader(shader.program, vertex_shader));
			gl_check!(gl::AttachShader(shader.program, fragment_shader));
			gl_check!(gl::LinkProgram(shader.program));
			gl_check!(gl::GetProgramiv(shader.program, gl::LINK_STATUS, &mut status));
			if status == 0 {
				let mut log_len = 0;
				gl_check!(gl::GetProgramiv(shader.program, gl::INFO_LOG_LENGTH, &mut log_len));
				let mut log = vec![0; log_len as usize];
				gl_check!(gl::GetProgramInfoLog(shader.program, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _));
				shader.compile_log.push_str("# Program link log:\n");
				shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
				success = false;
			}
			else {
				gl_check!(gl::UseProgram(shader.program));
				let mut count = 0;
				gl_check!(gl::GetProgramiv(shader.program, gl::ACTIVE_UNIFORMS, &mut count));
				for i in 0..count {
					let mut name_len = 0;
					let mut size = 0;
					let mut ty = 0;
					let mut name = [0; 64];
					gl_check!(gl::GetActiveUniform(shader.program, i as u32, 64, &mut name_len, &mut size, &mut ty, name.as_mut_ptr() as *mut _));
					shader.active_uniforms.push(GlShaderActiveUniform {
						location: i,
						namelen: name_len as u8,
						namebuf: name,
						_size: size,
						_ty: ty,
					});
				}
			}
		}

		gl_check!(gl::DeleteShader(vertex_shader));
		gl_check!(gl::DeleteShader(fragment_shader));
		return if success { Ok(()) } else { Err(crate::GfxError::ShaderCompileError) };
	}

	fn shader_compile_log(&mut self, id: crate::Shader) -> Result<String, crate::GfxError> {
		let Some(shader) = self.shaders.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(shader.compile_log.clone());
	}

	fn shader_delete(&mut self, id: crate::Shader, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(shader) = self.shaders.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteProgram(shader.program));
		Ok(())
	}

	fn texture2d_create(&mut self, name: Option<&str>, info: &crate::Texture2DInfo) -> Result<crate::Texture2D, crate::GfxError> {
		let mut texture = 0;
		gl_check!(gl::GenTextures(1, &mut texture));
		// gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture));
		// gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		let id = self.textures2d.insert(name, GlTexture2D { texture, info: *info });
		return Ok(id);
	}

	fn texture2d_find(&mut self, name: &str) -> Result<crate::Texture2D, crate::GfxError> {
		self.textures2d.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}

	fn texture2d_set_data(&mut self, id: crate::Texture2D, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
		let format = match texture.info.format {
			crate::TextureFormat::R8G8B8 => gl::RGB,
			crate::TextureFormat::R8G8B8A8 => gl::RGBA,
		};
		gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
		gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, texture.info.width, texture.info.height, 0, format, gl::UNSIGNED_BYTE, data.as_ptr() as *const _));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl_texture_wrap(texture.info.wrap_u) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl_texture_wrap(texture.info.wrap_v) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_texture_filter(texture.info.filter_mag) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_texture_filter(texture.info.filter_min) as gl::types::GLint));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
		Ok(())
	}

	fn texture2d_get_info(&mut self, id: crate::Texture2D) -> Result<crate::Texture2DInfo, crate::GfxError> {
		let Some(texture) = self.textures2d.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(texture.info);
	}

	fn texture2d_delete(&mut self, id: crate::Texture2D, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures2d.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteTextures(1, &texture.texture));
		Ok(())
	}

	fn texture2darray_create(&mut self, name: Option<&str>, info: &crate::Texture2DArrayInfo) -> Result<crate::Texture2DArray, crate::GfxError> {
		let mut texture = 0;
		gl_check!(gl::GenTextures(1, &mut texture));
		// gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture));
		// gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0));
		let id = self.textures2darray.insert(name, GlTexture2DArray { texture, info: *info });
		return Ok(id);
	}
	fn texture2darray_find(&mut self, name: &str) -> Result<crate::Texture2DArray, crate::GfxError> {
		self.textures2darray.find_id(name).ok_or(crate::GfxError::NameNotFound)
	}
	fn texture2darray_set_data(&mut self, id: crate::Texture2DArray, index: usize, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures2darray.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		if index >= texture.info.count as usize { return Err(crate::GfxError::IndexOutOfBounds) }
		gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture.texture));
		let format = match texture.info.format {
			crate::TextureFormat::R8G8B8 => gl::RGB,
			crate::TextureFormat::R8G8B8A8 => gl::RGBA,
		};
		gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
		gl_check!(gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, format as i32, texture.info.width, texture.info.height, index as i32, 0, format, gl::UNSIGNED_BYTE, data.as_ptr() as *const _));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl_texture_wrap(texture.info.wrap_u) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl_texture_wrap(texture.info.wrap_v) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl_texture_filter(texture.info.filter_mag) as gl::types::GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl_texture_filter(texture.info.filter_min) as gl::types::GLint));
		gl_check!(gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0));
		Ok(())
	}
	fn texture2darray_get_info(&mut self, id: crate::Texture2DArray) -> Result<crate::Texture2DArrayInfo, crate::GfxError> {
		let Some(texture) = self.textures2darray.get(id) else { return Err(crate::GfxError::InvalidHandle) };
		return Ok(texture.info);
	}
	fn texture2darray_delete(&mut self, id: crate::Texture2DArray, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures2darray.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		gl_check!(gl::DeleteTextures(1, &texture.texture));
		Ok(())
	}

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
			crate::SurfaceFormat::R8G8B8 => gl::RGB,
			crate::SurfaceFormat::R8G8B8A8 => gl::RGBA,
		};
		gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)); // Force 1 byte alignment
		gl_check!(gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, info.width, info.height, 0, format, gl::UNSIGNED_BYTE, std::ptr::null()));
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

	fn surface_delete(&mut self, id: crate::Surface, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(surface) = self.surfaces.remove(id, free_handle) else { return Err(crate::GfxError::InvalidHandle) };
		self.texture2d_delete(surface.texture, free_handle)?;
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
