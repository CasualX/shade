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

struct GlVertexBuffer {
	buffer: gl::types::GLuint,
	vao: gl::types::GLuint,
	_layout: &'static crate::VertexLayout,
	_count: usize,
}

impl Resource for GlVertexBuffer {
	type Handle = crate::VertexBuffer;
}

struct GlIndexBuffer {
	buffer: gl::types::GLuint,
	ty: gl::types::GLenum,
	_count: usize,
}

impl Resource for GlIndexBuffer {
	type Handle = crate::IndexBuffer;
}

struct GlUniformBuffer {
	data: Vec<u8>,
	layout: &'static crate::UniformLayout,
}

impl Resource for GlUniformBuffer {
	type Handle = crate::UniformBuffer;
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

impl Resource for GlShader {
	type Handle = crate::Shader;
}

struct GlTexture2D {
	texture: gl::types::GLuint,
	info: crate::Texture2DInfo,
}

impl Resource for GlTexture2D {
	type Handle = crate::Texture2D;
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
	check(|| unsafe { gl::Enable(gl::BLEND) });
	check(|| unsafe { gl::BlendFunc(p.sfactor, p.dfactor) });
	check(|| unsafe { gl::BlendEquation(p.equation) });
}

fn gl_scissor(scissor: &Option<cvmath::Rect<i32>>) {
	if let Some(scissor) = scissor {
		check(|| unsafe { gl::Enable(gl::SCISSOR_TEST) });
		check(|| unsafe { gl::Scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()) });
	}
	else {
		check(|| unsafe { gl::Disable(gl::SCISSOR_TEST) });
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
		unsafe { check(|| gl::Enable(gl::DEPTH_TEST)) };
		unsafe { check(|| gl::DepthFunc(func)) };
	}
	else {
		unsafe { check(|| gl::Disable(gl::DEPTH_TEST)) };
	}
}

fn gl_cull_face(cull_mode: Option<crate::CullMode>) {
	if let Some(cull_mode) = cull_mode {
		let mode = match cull_mode {
			crate::CullMode::CCW => gl::FRONT,
			crate::CullMode::CW => gl::BACK,
		};
		unsafe { check(|| gl::Enable(gl::CULL_FACE)) };
		unsafe { check(|| gl::CullFace(mode)) };
	}
	else {
		unsafe { check(|| gl::Disable(gl::CULL_FACE)) };
	}
}

#[inline]
fn gl_mat_order(order: crate::UniformMatOrder) -> gl::types::GLboolean {
	match order {
		crate::UniformMatOrder::ColumnMajor => gl::FALSE,
		crate::UniformMatOrder::RowMajor => gl::TRUE,
	}
}

fn gl_uniforms(ub: &GlUniformBuffer, shader: &GlShader, uniform_index: u32, textures: &ResourceMap<GlTexture2D>) {
	check(|| unsafe { gl::UseProgram(shader.program) });

	let data_ptr = unsafe { ub.data.as_ptr().add(ub.layout.size as usize * uniform_index as usize) };
	for uattr in ub.layout.attributes {
		let data_ptr = unsafe { data_ptr.offset(uattr.offset as isize) };
		if let Some(location) = shader.uniform_location(uattr.name) {
			// println!("Uniform: {} (index: {})", uattr.name, i);
			match uattr.ty {
				crate::UniformType::D1 => check(|| unsafe { gl::Uniform1dv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::D2 => check(|| unsafe { gl::Uniform2dv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::D3 => check(|| unsafe { gl::Uniform3dv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::D4 => check(|| unsafe { gl::Uniform4dv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::F1 => check(|| unsafe { gl::Uniform1fv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::F2 => check(|| unsafe { gl::Uniform2fv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::F3 => check(|| unsafe { gl::Uniform3fv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::F4 => check(|| unsafe { gl::Uniform4fv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::I1 => check(|| unsafe { gl::Uniform1iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::I2 => check(|| unsafe { gl::Uniform2iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::I3 => check(|| unsafe { gl::Uniform3iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::I4 => check(|| unsafe { gl::Uniform4iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::U1 => check(|| unsafe { gl::Uniform1uiv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::U2 => check(|| unsafe { gl::Uniform2uiv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::U3 => check(|| unsafe { gl::Uniform3uiv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::U4 => check(|| unsafe { gl::Uniform4uiv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::B1 => check(|| unsafe { gl::Uniform1iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::B2 => check(|| unsafe { gl::Uniform2iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::B3 => check(|| unsafe { gl::Uniform3iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::B4 => check(|| unsafe { gl::Uniform4iv(location, uattr.len as i32, data_ptr as *const _) }),
				crate::UniformType::Mat2x2 { order } => check(|| unsafe { gl::UniformMatrix2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat2x3 { order } => check(|| unsafe { gl::UniformMatrix2x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat2x4 { order } => check(|| unsafe { gl::UniformMatrix2x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat3x2 { order } => check(|| unsafe { gl::UniformMatrix3x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat3x3 { order } => check(|| unsafe { gl::UniformMatrix3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat3x4 { order } => check(|| unsafe { gl::UniformMatrix3x4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat4x2 { order } => check(|| unsafe { gl::UniformMatrix4x2fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat4x3 { order } => check(|| unsafe { gl::UniformMatrix4x3fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Mat4x4 { order } => check(|| unsafe { gl::UniformMatrix4fv(location, uattr.len as i32, gl_mat_order(order), data_ptr as *const _) }),
				crate::UniformType::Sampler2D(index) => {
					let id = unsafe { *(data_ptr as *const crate::Texture2D) };
					let texture = gl_texture_id(textures, id);
					check(|| unsafe { gl::ActiveTexture(gl::TEXTURE0 + index as u32) });
					check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, texture) });
				}
			}
		}
		else {
			// panic!("Uniform not found: {}", uattr.name);
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
	vertices: ResourceMap<GlVertexBuffer>,
	indices: ResourceMap<GlIndexBuffer>,
	uniforms: ResourceMap<GlUniformBuffer>,
	shaders: ResourceMap<GlShader>,
	textures: ResourceMap<GlTexture2D>,
	surfaces: ResourceMap<GlSurface>,
	drawing: bool,
}

impl GlGraphics {
	pub fn new() -> Self {
		GlGraphics {
			vertices: ResourceMap::new(),
			indices: ResourceMap::new(),
			uniforms: ResourceMap::new(),
			shaders: ResourceMap::new(),
			textures: ResourceMap::new(),
			surfaces: ResourceMap::new(),
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
			check(|| unsafe { gl::Enable(gl::SCISSOR_TEST) });
			check(|| unsafe { gl::Scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()) });
		}
		else {
			check(|| unsafe { gl::Disable(gl::SCISSOR_TEST) });
		}

		let mut mask = 0;
		if let Some(color) = args.color {
			check(|| unsafe { gl::ClearColor(color.x, color.y, color.z, color.w) });
			mask |= gl::COLOR_BUFFER_BIT;
		}
		if let Some(depth) = args.depth {
			check(|| unsafe { gl::ClearDepth(depth as f64) });
			mask |= gl::DEPTH_BUFFER_BIT;
		}
		if let Some(stencil) = args.stencil{
			check(|| unsafe { gl::ClearStencil(stencil as i32) });
			mask |= gl::STENCIL_BUFFER_BIT;
		}
		check(|| unsafe { gl::Clear(mask) });

		Ok(())
	}

	fn draw(&mut self, args: &crate::DrawArgs) -> Result<(), crate::GfxError> {
		if !self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		let this = &*self;
		let Some(vb) = this.vertices.get(args.vertices) else { return Err(crate::GfxError::InvalidVertexBufferHandle) };
		let Some(ub) = self.uniforms.get(args.uniforms) else { return Err(crate::GfxError::InvalidUniformBufferHandle) };
		let Some(shader) = this.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidShaderHandle) };

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
		check(|| unsafe { gl::Viewport(args.viewport.mins.x, args.viewport.mins.y, args.viewport.width(), args.viewport.height()) });

		check(|| unsafe { gl::BindVertexArray(vb.vao) });

		gl_uniforms(ub, shader, args.uniform_index, &self.textures);

		let mode = match args.prim_type {
			crate::PrimType::Lines => gl::LINES,
			crate::PrimType::Triangles => gl::TRIANGLES,
		};
		if args.instances >= 0 {
			check(|| unsafe { gl::DrawArraysInstanced(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32, args.instances) });
		}
		else {
			check(|| unsafe { gl::DrawArrays(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32) });
		}

		Ok(())
	}

	fn draw_indexed(&mut self, args: &crate::DrawIndexedArgs) -> Result<(), crate::GfxError> {
		if !self.drawing {
			return Err(crate::GfxError::InvalidDrawCallTime);
		}

		let this = &*self;
		let Some(vb) = this.vertices.get(args.vertices) else { return Err(crate::GfxError::InvalidVertexBufferHandle) };
		let Some(ib) = this.indices.get(args.indices) else { return Err(crate::GfxError::InvalidIndexBufferHandle) };
		let Some(ub) = self.uniforms.get(args.uniforms) else { return Err(crate::GfxError::InvalidUniformBufferHandle) };
		let Some(shader) = this.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidShaderHandle) };

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
		check(|| unsafe { gl::Viewport(args.viewport.mins.x, args.viewport.mins.y, args.viewport.width(), args.viewport.height()) });

		check(|| unsafe { gl::BindVertexArray(vb.vao) });
		check(|| unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.buffer) });

		gl_uniforms(ub, shader, args.uniform_index, &self.textures);

		let mode = match args.prim_type {
			crate::PrimType::Lines => gl::LINES,
			crate::PrimType::Triangles => gl::TRIANGLES,
		};
		let count = args.index_end - args.index_start;
		let offset = match ib.ty {
			gl::UNSIGNED_INT => args.index_start * 4,
			gl::UNSIGNED_SHORT => args.index_start * 2,
			_ => args.index_start,
		};
		if args.instances >= 0 {
			check(|| unsafe { gl::DrawElementsInstanced(mode, count as i32, ib.ty, offset as *const _, args.instances) });
		}
		else {
			check(|| unsafe { gl::DrawElements(mode, count as i32, ib.ty, offset as *const _) });
		}

		Ok(())
	}

	fn end(&mut self) -> Result<(), crate::GfxError> {
		self.drawing = false;
		Ok(())
	}

	fn vertex_buffer_create(&mut self, name: Option<&str>, _layout: &'static crate::VertexLayout, _count: usize) -> Result<crate::VertexBuffer, crate::GfxError> {
		let mut buffer = 0;
		let mut vao = 0;
		check(|| unsafe { gl::GenBuffers(1, &mut buffer) });
		check(|| unsafe { gl::GenVertexArrays(1, &mut vao) });

		check(|| unsafe { gl::BindVertexArray(vao) });
		check(|| unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, buffer) });

		for (i, attr) in _layout.attributes.iter().enumerate() {
			let (type_, normalized) = match attr.format {
				crate::VertexAttributeFormat::F32 => (gl::FLOAT, false),
				crate::VertexAttributeFormat::F64 => (gl::DOUBLE, false),
				crate::VertexAttributeFormat::I8 => (gl::BYTE, false),
				crate::VertexAttributeFormat::U8 => (gl::UNSIGNED_BYTE, false),
				crate::VertexAttributeFormat::I16 => (gl::SHORT, false),
				crate::VertexAttributeFormat::U16 => (gl::UNSIGNED_SHORT, false),
				crate::VertexAttributeFormat::I32 => (gl::INT, false),
				crate::VertexAttributeFormat::U32 => (gl::UNSIGNED_INT, false),
				crate::VertexAttributeFormat::I8Norm => (gl::BYTE, true),
				crate::VertexAttributeFormat::U8Norm => (gl::UNSIGNED_BYTE, true),
				crate::VertexAttributeFormat::I16Norm => (gl::SHORT, true),
				crate::VertexAttributeFormat::U16Norm => (gl::UNSIGNED_SHORT, true),
			};
			check(|| unsafe { gl::VertexAttribPointer(i as u32, attr.len as i32, type_, normalized as u8, _layout.size as i32, attr.offset as usize as *const _) });
			check(|| unsafe { gl::EnableVertexAttribArray(i as u32) });
		}

		check(|| unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) });
		check(|| unsafe { gl::BindVertexArray(0) });

		let id = self.vertices.insert(name, GlVertexBuffer { buffer, vao, _layout, _count });
		return Ok(id);
	}

	fn vertex_buffer_find(&mut self, name: &str) -> Result<crate::VertexBuffer, crate::GfxError> {
		let Some(id) = self.vertices.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}

	fn vertex_buffer_set_data(&mut self, id: crate::VertexBuffer, data: &[u8], usage: crate::BufferUsage) -> Result<(), crate::GfxError> {
		let Some(vb) = self.vertices.get_mut(id) else { return Err(crate::GfxError::InvalidVertexBufferHandle) };
		let size = mem::size_of_val(data) as gl::types::GLsizeiptr;
		let gl_usage = match usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		check(|| unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, vb.buffer) });
		check(|| unsafe { gl::BufferData(gl::ARRAY_BUFFER, size, data.as_ptr() as *const _, gl_usage) });
		check(|| unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) });
		Ok(())
	}

	fn vertex_buffer_delete(&mut self, id: crate::VertexBuffer, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(vb) = self.vertices.remove(id, free_handle) else { return Err(crate::GfxError::InvalidVertexBufferHandle) };
		check(|| unsafe { gl::DeleteBuffers(1, &vb.buffer) });
		Ok(())
	}

	fn index_buffer_create(&mut self, name: Option<&str>, count: usize) -> Result<crate::IndexBuffer, crate::GfxError> {
		let mut buffer = 0;
		check(|| unsafe { gl::GenBuffers(1, &mut buffer) });
		check(|| unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer) });
		check(|| unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) });
		let id = self.indices.insert(name, GlIndexBuffer { buffer, ty: gl::UNSIGNED_INT, _count: count });
		return Ok(id);
	}

	fn index_buffer_find(&mut self, name: &str) -> Result<crate::IndexBuffer, crate::GfxError> {
		let Some(id) = self.indices.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}

	fn index_buffer_set_data(&mut self, id: crate::IndexBuffer, data: &[u32], usage: crate::BufferUsage) -> Result<(), crate::GfxError> {
		let Some(ib) = self.indices.get(id) else { return Err(crate::GfxError::InvalidIndexBufferHandle) };
		let size = mem::size_of_val(data) as gl::types::GLsizeiptr;
		let usage = match usage {
			crate::BufferUsage::Static => gl::STATIC_DRAW,
			crate::BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
			crate::BufferUsage::Stream => gl::STREAM_DRAW,
		};
		check(|| unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.buffer) });
		check(|| unsafe { gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data.as_ptr() as *const _, usage) });
		check(|| unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) });
		Ok(())
	}

	fn index_buffer_delete(&mut self, id: crate::IndexBuffer, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(ib) = self.indices.remove(id, free_handle) else { return Err(crate::GfxError::InvalidIndexBufferHandle) };
		check(|| unsafe { gl::DeleteBuffers(1, &ib.buffer) });
		Ok(())
	}

	fn uniform_buffer_create(&mut self, name: Option<&str>, layout: &'static crate::UniformLayout, count: usize) -> Result<crate::UniformBuffer, crate::GfxError> {
		let data = Vec::with_capacity(layout.size as usize * count);
		let id = self.uniforms.insert(name, GlUniformBuffer { data, layout });
		return Ok(id);
	}
	fn uniform_buffer_find(&mut self, name: &str) -> Result<crate::UniformBuffer, crate::GfxError> {
		let Some(id) = self.uniforms.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}
	fn uniform_buffer_set_data(&mut self, id: crate::UniformBuffer, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(ub) = self.uniforms.get_mut(id) else { return Err(crate::GfxError::InvalidUniformBufferHandle) };
		ub.data.clear();
		ub.data.extend_from_slice(data);
		Ok(())
	}
	fn uniform_buffer_delete(&mut self, id: crate::UniformBuffer, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(_) = self.uniforms.remove(id, free_handle) else { return Err(crate::GfxError::InvalidUniformBufferHandle) };
		Ok(())
	}

	fn shader_create(&mut self, name: Option<&str>) -> Result<crate::Shader, crate::GfxError> {
		let program = check(|| unsafe { gl::CreateProgram() });
		let id = self.shaders.insert(name, GlShader { program, compile_log: String::new(), active_uniforms: Vec::new() });
		return Ok(id);
	}

	fn shader_find(&mut self, name: &str) -> Result<crate::Shader, crate::GfxError> {
		let Some(id) = self.shaders.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}

	fn shader_compile(&mut self, id: crate::Shader, vertex_source: &str, fragment_source: &str) -> Result<(), crate::GfxError> {
		let Some(shader) = self.shaders.get_mut(id) else { return Err(crate::GfxError::InvalidShaderHandle) };
		let mut success = true;
		let mut status = 0;

		shader.active_uniforms.clear();

		let vertex_shader = check(|| unsafe { gl::CreateShader(gl::VERTEX_SHADER) });
		check(|| unsafe { gl::ShaderSource(vertex_shader, 1, &(vertex_source.as_ptr() as *const _), &(vertex_source.len() as gl::types::GLint)) });
		check(|| unsafe { gl::CompileShader(vertex_shader) });
		check(|| unsafe { gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut status) });
		if status == 0 {
			let mut log_len = 0;
			check(|| unsafe { gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_len) });
			let mut log = vec![0; log_len as usize];
			check(|| unsafe { gl::GetShaderInfoLog(vertex_shader, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _) });
			shader.compile_log.push_str("# Vertex shader compile log:\n");
			shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
			success = false;
		}

		let fragment_shader = check(|| unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) });
		check(|| unsafe { gl::ShaderSource(fragment_shader, 1, &(fragment_source.as_ptr() as *const _), &(fragment_source.len() as gl::types::GLint)) });
		check(|| unsafe { gl::CompileShader(fragment_shader) });
		check(|| unsafe { gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut status) });
		if status == 0 {
			let mut log_len = 0;
			check(|| unsafe { gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_len) });
			let mut log = vec![0; log_len as usize];
			check(|| unsafe { gl::GetShaderInfoLog(fragment_shader, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _) });
			shader.compile_log.push_str("# Fragment shader compile log:\n");
			shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
			success = false;
		}

		if success {
			check(|| unsafe { gl::AttachShader(shader.program, vertex_shader) });
			check(|| unsafe { gl::AttachShader(shader.program, fragment_shader) });
			check(|| unsafe { gl::LinkProgram(shader.program) });
			check(|| unsafe { gl::GetProgramiv(shader.program, gl::LINK_STATUS, &mut status) });
			if status == 0 {
				let mut log_len = 0;
				check(|| unsafe { gl::GetProgramiv(shader.program, gl::INFO_LOG_LENGTH, &mut log_len) });
				let mut log = vec![0; log_len as usize];
				check(|| unsafe { gl::GetProgramInfoLog(shader.program, log_len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _) });
				shader.compile_log.push_str("# Program link log:\n");
				shader.compile_log.push_str(String::from_utf8_lossy(&log).as_ref());
				success = false;
			}
			else {
				check(|| unsafe { gl::UseProgram(shader.program) });
				let mut count = 0;
				check(|| unsafe { gl::GetProgramiv(shader.program, gl::ACTIVE_UNIFORMS, &mut count) });
				for i in 0..count {
					let mut name_len = 0;
					let mut size = 0;
					let mut ty = 0;
					let mut name = [0; 64];
					check(|| unsafe { gl::GetActiveUniform(shader.program, i as u32, 64, &mut name_len, &mut size, &mut ty, name.as_mut_ptr() as *mut _) });
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

		check(|| unsafe { gl::DeleteShader(vertex_shader) });
		check(|| unsafe { gl::DeleteShader(fragment_shader) });
		return if success { Ok(()) } else { Err(crate::GfxError::ShaderCompileError) };
	}

	fn shader_compile_log(&mut self, id: crate::Shader) -> Result<String, crate::GfxError> {
		let Some(shader) = self.shaders.get(id) else { return Err(crate::GfxError::InvalidShaderHandle) };
		return Ok(shader.compile_log.clone());
	}

	fn shader_delete(&mut self, id: crate::Shader, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(shader) = self.shaders.remove(id, free_handle) else { return Err(crate::GfxError::InvalidShaderHandle) };
		check(|| unsafe { gl::DeleteProgram(shader.program) });
		Ok(())
	}

	fn texture2d_create(&mut self, name: Option<&str>, info: &crate::Texture2DInfo) -> Result<crate::Texture2D, crate::GfxError> {
		let mut texture = 0;
		check(|| unsafe { gl::GenTextures(1, &mut texture) });
		// check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, texture) });
		// check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) });
		let id = self.textures.insert(name, GlTexture2D { texture, info: *info });
		return Ok(id);
	}

	fn texture2d_find(&mut self, name: &str) -> Result<crate::Texture2D, crate::GfxError> {
		let Some(id) = self.textures.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}

	fn texture2d_set_data(&mut self, id: crate::Texture2D, data: &[u8]) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures.get(id) else { return Err(crate::GfxError::InvalidTexture2DHandle) };
		check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, texture.texture) });
		check(|| unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, texture.info.width, texture.info.height, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const _) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl_texture_wrap(texture.info.wrap_u) as gl::types::GLint) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl_texture_wrap(texture.info.wrap_v) as gl::types::GLint) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_texture_filter(texture.info.filter_mag) as gl::types::GLint) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_texture_filter(texture.info.filter_min) as gl::types::GLint) });
		check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) });
		Ok(())
	}

	fn texture2d_get_info(&mut self, id: crate::Texture2D) -> Result<crate::Texture2DInfo, crate::GfxError> {
		let Some(texture) = self.textures.get(id) else { return Err(crate::GfxError::InvalidTexture2DHandle) };
		return Ok(texture.info);
	}

	fn texture2d_delete(&mut self, id: crate::Texture2D, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(texture) = self.textures.remove(id, free_handle) else { return Err(crate::GfxError::InvalidTexture2DHandle) };
		check(|| unsafe { gl::DeleteTextures(1, &texture.texture) });
		Ok(())
	}

	fn surface_create(&mut self, name: Option<&str>, info: &crate::SurfaceInfo) -> Result<crate::Surface, crate::GfxError> {
		let texture = Handle::create(0);

		let mut frame_buf = 0;
		let mut depth_buf = 0;
		let mut tex_buf = 0;
		check(|| unsafe { gl::GenFramebuffers(1, &mut frame_buf) });
		check(|| unsafe { gl::GenRenderbuffers(1, &mut depth_buf) });
		check(|| unsafe { gl::GenTextures(1, &mut tex_buf) });

		check(|| unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buf) });

		check(|| unsafe { gl::BindRenderbuffer(gl::RENDERBUFFER, depth_buf) });
		check(|| unsafe { gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, info.width, info.height) });
		check(|| unsafe { gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, depth_buf) });

		check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, tex_buf) });

		check(|| unsafe { gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, info.width, info.height, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null()) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32) });
		check(|| unsafe { gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32) });

		check(|| unsafe { gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_buf, 0) });

		check(|| unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) });
		check(|| unsafe { gl::BindRenderbuffer(gl::RENDERBUFFER, 0) });
		check(|| unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) });

		// let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
		// if status != gl::FRAMEBUFFER_COMPLETE {
		// 	panic!("Framebuffer is not complete: {}", status);
		// }

		let id = self.surfaces.insert(name, GlSurface { texture, frame_buf, depth_buf, tex_buf, format: info.format, width: info.width, height: info.height });
		return Ok(id);
	}

	fn surface_find(&mut self, name: &str) -> Result<crate::Surface, crate::GfxError> {
		let Some(id) = self.surfaces.find_id(name) else { return Err(crate::GfxError::NameNotFound) };
		return Ok(id);
	}

	fn surface_get_info(&mut self, id: crate::Surface) -> Result<crate::SurfaceInfo, crate::GfxError> {
		let Some(surface) = self.surfaces.get(id) else { return Err(crate::GfxError::InvalidSurfaceHandle) };
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
		let Some(surface) = self.surfaces.get(id) else { return Err(crate::GfxError::InvalidSurfaceHandle) };
		return Ok(surface.texture);
	}

	fn surface_delete(&mut self, id: crate::Surface, free_handle: bool) -> Result<(), crate::GfxError> {
		let Some(surface) = self.surfaces.remove(id, free_handle) else { return Err(crate::GfxError::InvalidSurfaceHandle) };
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
	{
		let error = unsafe { gl::GetError() };
		if error != gl::NO_ERROR {
			panic!("OpenGL error: {}", error);
		}
	}
	result
}

#[cfg(not(debug_assertions))]
#[inline]
fn check<T, F: FnOnce() -> T>(f: F) -> T {
	f()
}
