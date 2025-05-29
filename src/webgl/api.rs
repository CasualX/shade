pub mod types {
	pub type GLenum = u32;
	pub type GLint = i32;
	pub type GLuint = u32;
	pub type GLsizei = i32;
	pub type GLbitfield = u32;
	pub type GLboolean = bool;
	pub type GLintptr = isize;
	pub type GLsizeiptr = isize;
}

#[link(wasm_import_module = "webgl")]
extern "C" {
	pub fn consoleLog(message_ptr: *const u8, message_len: usize);
	pub fn enable(cap: types::GLenum);
	pub fn disable(cap: types::GLenum);
	pub fn scissor(x: types::GLint, y: types::GLint, width: types::GLsizei, height: types::GLsizei);
	pub fn blendFunc(sfactor: types::GLenum, dfactor: types::GLenum);
	pub fn blendEquation(mode: types::GLenum);
	pub fn depthFunc(func: types::GLenum);
	pub fn cullFace(mode: types::GLenum);
	pub fn clearColor(red: f32, green: f32, blue: f32, alpha: f32);
	pub fn clearDepth(depth: f64);
	pub fn clearStencil(s: types::GLint);
	pub fn clear(mask: types::GLbitfield);
	pub fn viewport(x: types::GLint, y: types::GLint, width: types::GLsizei, height: types::GLsizei);
	pub fn createBuffer() -> types::GLuint;
	pub fn bindBuffer(target: types::GLenum, buffer: types::GLuint);
	pub fn deleteBuffer(buffer: types::GLuint);
	pub fn bufferData(target: types::GLenum, size: types::GLsizeiptr, data: *const u8, usage: types::GLenum);
	pub fn enableVertexAttribArray(index: types::GLuint);
	// pub fn disableVertexAttribArray(index: types::GLuint);
	pub fn vertexAttribPointer(
		index: types::GLuint,
		size: types::GLint,
		type_: types::GLenum,
		normalized: types::GLboolean,
		stride: types::GLsizei,
		offset: types::GLintptr,
	);
	pub fn createProgram() -> types::GLuint;
	pub fn deleteProgram(program: types::GLuint);
	pub fn createShader(shader_type: types::GLenum) -> types::GLuint;
	pub fn deleteShader(shader: types::GLuint);
	pub fn shaderSource(shader: types::GLuint, source_ptr: *const u8, source_len: usize);
	pub fn compileShader(shader: types::GLuint);
	pub fn getShaderParameter(shader: types::GLuint, pname: types::GLenum) -> types::GLint;
	pub fn getShaderInfoLog(shader: types::GLuint);
	pub fn attachShader(program: types::GLuint, shader: types::GLuint);
	pub fn linkProgram(program: types::GLuint);
	pub fn useProgram(program: types::GLuint);
	pub fn getProgramParameter(program: types::GLuint, pname: types::GLenum) -> types::GLint;
	pub fn getProgramInfoLog(program: types::GLuint);
	pub fn getActiveUniform(rogram: types::GLuint, index: types::GLuint, bufSize: types::GLsizei, length: *mut types::GLsizei, size: *mut types::GLint, type_: *mut types::GLenum, name: *mut u8);
	pub fn getAttribLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	// pub fn getUniformLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	pub fn uniform1fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform2fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform3fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform4fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform1iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform2iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform3iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform4iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniformMatrix2fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix2x3fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix2x4fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix3x2fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix3fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix3x4fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix4x2fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix4x3fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix4fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn activeTexture(texture: types::GLenum);
	pub fn bindTexture(target: types::GLenum, texture: types::GLuint);
	pub fn drawArrays(mode: types::GLenum, first: types::GLint, count: types::GLsizei);
	pub fn drawArraysInstanced(mode: types::GLenum, first: types::GLint, count: types::GLsizei, instancecount: types::GLsizei);
	pub fn drawElements(mode: types::GLenum, count: types::GLsizei, type_: types::GLenum, indices: types::GLintptr);
	pub fn drawElementsInstanced(mode: types::GLenum, count: types::GLsizei, type_: types::GLenum, indices: types::GLintptr, instancecount: types::GLsizei);
}

pub const FALSE: types::GLboolean = false;
pub const TRUE: types::GLboolean = true;

pub const ZERO: types::GLenum = 0;
pub const ONE: types::GLenum = 1;
pub const LINES: types::GLenum = 1;
pub const TRIANGLES: types::GLenum = 4;
pub const NEVER: types::GLenum = 512;
pub const LESS: types::GLenum = 513;
pub const EQUAL: types::GLenum = 514;
pub const LEQUAL: types::GLenum = 515;
pub const GREATER: types::GLenum = 516;
pub const NOTEQUAL: types::GLenum = 517;
pub const GEQUAL: types::GLenum = 518;
pub const ALWAYS: types::GLenum = 519;
pub const SRC_ALPHA: types::GLenum = 770;
pub const ONE_MINUS_SRC_ALPHA: types::GLenum = 771;
pub const DST_COLOR: types::GLenum = 774;
pub const ONE_MINUS_SRC_COLOR: types::GLenum = 769;

pub const FRONT: types::GLenum = 1028;
pub const BACK: types::GLenum = 1029;
pub const CULL_FACE: types::GLenum = 2884;
pub const DEPTH_TEST: types::GLenum = 2929;
pub const BLEND: types::GLenum = 3042;
pub const SCISSOR_TEST: types::GLenum = 3089;
pub const TEXTURE_2D: types::GLenum = 3553;
pub const BYTE: types::GLenum = 5120;
pub const UNSIGNED_BYTE: types::GLenum = 5121;
pub const SHORT: types::GLenum = 5122;
pub const UNSIGNED_SHORT: types::GLenum = 5123;
pub const UNSIGNED_INT: types::GLenum = 5125;
pub const FLOAT: types::GLenum = 5126;

pub const FUNC_ADD: types::GLenum = 32774;
pub const MIN: types::GLenum = 32775;
pub const MAX: types::GLenum = 32776;
pub const TEXTURE0: types::GLenum = 33984;
pub const ARRAY_BUFFER: types::GLenum = 34962;
pub const ELEMENT_ARRAY_BUFFER: types::GLenum = 34963;

pub const STREAM_DRAW: types::GLenum = 35040;
pub const STATIC_DRAW: types::GLenum = 35044;
pub const DYNAMIC_DRAW: types::GLenum = 35048;

pub const FRAGMENT_SHADER: types::GLenum = 35632;
pub const VERTEX_SHADER: types::GLenum = 35633;
pub const COMPILE_STATUS: types::GLenum = 35713;
pub const LINK_STATUS: types::GLenum = 35714;
pub const ACTIVE_UNIFORMS: types::GLenum = 35718;
pub const ACTIVE_ATTRIBUTES: types::GLenum = 35721;

pub const COLOR_BUFFER_BIT: types::GLbitfield = 16384;
pub const DEPTH_BUFFER_BIT: types::GLbitfield = 256;
pub const STENCIL_BUFFER_BIT: types::GLbitfield = 1024;
