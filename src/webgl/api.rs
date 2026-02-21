#![allow(dead_code)]
#![allow(non_upper_case_globals)]

pub mod types {
	pub type GLenum = u32;
	pub type GLint = i32;
	pub type GLuint = u32;
	pub type GLsizei = i32;
	pub type GLbitfield = u32;
	pub type GLboolean = bool;
	pub type GLintptr = isize;
	pub type GLsizeiptr = isize;
	pub type GLfloat = f32;
}

#[cfg(target_family = "wasm")]
#[link(wasm_import_module = "webgl")]
extern "C" {
	pub fn consoleLog(message_ptr: *const u8, message_len: usize);
	pub fn now() -> f64;
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
	pub fn colorMask(red: types::GLboolean, green: types::GLboolean, blue: types::GLboolean, alpha: types::GLboolean);
	pub fn depthMask(flag: types::GLboolean);
	pub fn stencilMask(mask: types::GLuint);
	pub fn viewport(x: types::GLint, y: types::GLint, width: types::GLsizei, height: types::GLsizei);
	pub fn createBuffer() -> types::GLuint;
	pub fn bindBuffer(target: types::GLenum, buffer: types::GLuint);
	pub fn deleteBuffer(buffer: types::GLuint);
	pub fn bufferData(target: types::GLenum, size: types::GLsizeiptr, data: *const u8, usage: types::GLenum);
	pub fn bufferSubData(target: types::GLenum, offset: types::GLintptr, size: types::GLsizeiptr, data: *const u8);
	pub fn enableVertexAttribArray(index: types::GLuint);
	pub fn disableVertexAttribArray(index: types::GLuint);
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
	pub fn getUniformLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	pub fn getActiveAttrib(program: types::GLuint, index: types::GLuint, bufSize: types::GLsizei, length: *mut types::GLsizei, size: *mut types::GLint, type_: *mut types::GLenum, name: *mut u8);
	pub fn getAttribLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	pub fn uniform1fv(location: types::GLuint, count: types::GLsizei, value: *const f32);
	pub fn uniform2fv(location: types::GLuint, count: types::GLsizei, value: *const [f32; 2]);
	pub fn uniform3fv(location: types::GLuint, count: types::GLsizei, value: *const [f32; 3]);
	pub fn uniform4fv(location: types::GLuint, count: types::GLsizei, value: *const [f32; 4]);
	pub fn uniform1iv(location: types::GLuint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform2iv(location: types::GLuint, count: types::GLsizei, value: *const [types::GLint; 2]);
	pub fn uniform3iv(location: types::GLuint, count: types::GLsizei, value: *const [types::GLint; 3]);
	pub fn uniform4iv(location: types::GLuint, count: types::GLsizei, value: *const [types::GLint; 4]);
	pub fn uniformMatrix2fv(location: types::GLuint, count: types::GLsizei, transpose: types::GLboolean, value: *const [[f32; 2]; 2]);
	pub fn uniformMatrix3fv(location: types::GLuint, count: types::GLsizei, transpose: types::GLboolean, value: *const [[f32; 3]; 3]);
	pub fn uniformMatrix4fv(location: types::GLuint, count: types::GLsizei, transpose: types::GLboolean, value: *const [[f32; 4]; 4]);
	pub fn vertexAttribDivisor(index: types::GLuint, divisor: types::GLuint);
	pub fn createTexture() -> types::GLuint;
	pub fn deleteTexture(texture: types::GLuint);
	pub fn activeTexture(texture: types::GLenum);
	pub fn bindTexture(target: types::GLenum, texture: types::GLuint);
	pub fn generateMipmap(target: types::GLenum);
	pub fn pixelStorei(pname: types::GLenum, param: types::GLint);
	pub fn texParameteri(target: types::GLenum, pname: types::GLenum, param: types::GLint);
	pub fn texStorage2D(target: types::GLenum, levels: types::GLsizei, internalformat: types::GLenum, width: types::GLsizei, height: types::GLsizei);
	pub fn texImage2D(
		target: types::GLenum,
		level: types::GLint,
		internalformat: types::GLenum,
		width: types::GLsizei,
		height: types::GLsizei,
		border: types::GLint,
		format: types::GLenum,
		type_: types::GLenum,
		pixels_ptr: *const u8,
		pixels_len: usize,
	);
	pub fn texSubImage2D(
		target: types::GLenum,
		level: types::GLint,
		xoffset: types::GLint,
		yoffset: types::GLint,
		width: types::GLsizei,
		height: types::GLsizei,
		format: types::GLenum,
		type_: types::GLenum,
		pixels_ptr: *const u8,
		pixels_len: usize,
	);
	pub fn createFramebuffer() -> types::GLuint;
	pub fn deleteFramebuffer(framebuffer: types::GLuint);
	pub fn bindFramebuffer(target: types::GLenum, framebuffer: types::GLuint);
	pub fn framebufferTexture2D(target: types::GLenum, attachment: types::GLenum, textarget: types::GLenum, texture: types::GLuint, level: types::GLint);
	pub fn drawBuffers(n: types::GLsizei, bufs: *const types::GLenum);
	pub fn readBuffer(src: types::GLenum);
	pub fn checkFramebufferStatus(target: types::GLenum) -> types::GLenum;
	pub fn readPixels(x: types::GLint, y: types::GLint, width: types::GLsizei, height: types::GLsizei, format: types::GLenum, type_: types::GLenum, pixels_ptr: *mut u8, pixels_len: usize);
	pub fn drawArrays(mode: types::GLenum, first: types::GLint, count: types::GLsizei);
	pub fn drawElements(mode: types::GLenum, count: types::GLsizei, type_: types::GLenum, indices: types::GLintptr);
	pub fn drawArraysInstanced(mode: types::GLenum, first: types::GLint, count: types::GLsizei, instancecount: types::GLsizei);
	pub fn drawElementsInstanced(mode: types::GLenum, count: types::GLsizei, type_: types::GLenum, indices: types::GLintptr, instancecount: types::GLsizei);
}

#[cfg(not(target_family = "wasm"))]
mod stubs {
	#![allow(non_snake_case)]
	use super::types::*;

	pub unsafe fn consoleLog(_message_ptr: *const u8, _message_len: usize) {}
	pub unsafe fn now() -> f64 { 0.0 }

	pub unsafe fn enable(_cap: GLenum) {}
	pub unsafe fn disable(_cap: GLenum) {}
	pub unsafe fn scissor(_x: GLint, _y: GLint, _width: GLsizei, _height: GLsizei) {}
	pub unsafe fn blendFunc(_sfactor: GLenum, _dfactor: GLenum) {}
	pub unsafe fn blendEquation(_mode: GLenum) {}
	pub unsafe fn depthFunc(_func: GLenum) {}
	pub unsafe fn cullFace(_mode: GLenum) {}
	pub unsafe fn clearColor(_red: f32, _green: f32, _blue: f32, _alpha: f32) {}
	pub unsafe fn clearDepth(_depth: f64) {}
	pub unsafe fn clearStencil(_s: GLint) {}
	pub unsafe fn clear(_mask: GLbitfield) {}
	pub unsafe fn colorMask(_red: GLboolean, _green: GLboolean, _blue: GLboolean, _alpha: GLboolean) {}
	pub unsafe fn depthMask(_flag: GLboolean) {}
	pub unsafe fn stencilMask(_mask: GLuint) {}
	pub unsafe fn viewport(_x: GLint, _y: GLint, _width: GLsizei, _height: GLsizei) {}

	pub unsafe fn createBuffer() -> GLuint { 0 }
	pub unsafe fn bindBuffer(_target: GLenum, _buffer: GLuint) {}
	pub unsafe fn deleteBuffer(_buffer: GLuint) {}
	pub unsafe fn bufferData(_target: GLenum, _size: GLsizeiptr, _data: *const u8, _usage: GLenum) {}
	pub unsafe fn bufferSubData(_target: GLenum, _offset: GLintptr, _size: GLsizeiptr, _data: *const u8) {}

	pub unsafe fn enableVertexAttribArray(_index: GLuint) {}
	pub unsafe fn disableVertexAttribArray(_index: GLuint) {}
	pub unsafe fn vertexAttribPointer(_index: GLuint, _size: GLint, _type_: GLenum, _normalized: GLboolean, _stride: GLsizei, _offset: GLintptr) {}

	pub unsafe fn createProgram() -> GLuint { 0 }
	pub unsafe fn deleteProgram(_program: GLuint) {}

	pub unsafe fn createShader(_shader_type: GLenum) -> GLuint { 0 }
	pub unsafe fn deleteShader(_shader: GLuint) {}
	pub unsafe fn shaderSource(_shader: GLuint, _source_ptr: *const u8, _source_len: usize) {}
	pub unsafe fn compileShader(_shader: GLuint) {}
	pub unsafe fn getShaderParameter(_shader: GLuint, _pname: GLenum) -> GLint { 0 }
	pub unsafe fn getShaderInfoLog(_shader: GLuint) {}
	pub unsafe fn attachShader(_program: GLuint, _shader: GLuint) {}
	pub unsafe fn linkProgram(_program: GLuint) {}
	pub unsafe fn useProgram(_program: GLuint) {}
	pub unsafe fn getProgramParameter(_program: GLuint, _pname: GLenum) -> GLint { 0 }
	pub unsafe fn getProgramInfoLog(_program: GLuint) {}
	pub unsafe fn getActiveUniform(_program: GLuint, _index: GLuint, _bufSize: GLsizei, _length: *mut GLsizei, _size: *mut GLint, _type_: *mut GLenum, _name: *mut u8) {}
	pub unsafe fn getUniformLocation(_program: GLuint, _name_ptr: *const u8, _name_len: usize) -> GLint { -1 }
	pub unsafe fn getActiveAttrib(_program: GLuint, _index: GLuint, _bufSize: GLsizei, _length: *mut GLsizei, _size: *mut GLint, _type_: *mut GLenum, _name: *mut u8) {}
	pub unsafe fn getAttribLocation(_program: GLuint, _name_ptr: *const u8, _name_len: usize) -> GLint { -1 }

	pub unsafe fn uniform1fv(_location: GLuint, _count: GLsizei, _value: *const f32) {}
	pub unsafe fn uniform2fv(_location: GLuint, _count: GLsizei, _value: *const [f32; 2]) {}
	pub unsafe fn uniform3fv(_location: GLuint, _count: GLsizei, _value: *const [f32; 3]) {}
	pub unsafe fn uniform4fv(_location: GLuint, _count: GLsizei, _value: *const [f32; 4]) {}
	pub unsafe fn uniform1iv(_location: GLuint, _count: GLsizei, _value: *const GLint) {}
	pub unsafe fn uniform2iv(_location: GLuint, _count: GLsizei, _value: *const [GLint; 2]) {}
	pub unsafe fn uniform3iv(_location: GLuint, _count: GLsizei, _value: *const [GLint; 3]) {}
	pub unsafe fn uniform4iv(_location: GLuint, _count: GLsizei, _value: *const [GLint; 4]) {}
	pub unsafe fn uniformMatrix2fv(_location: GLuint, _count: GLsizei, _transpose: GLboolean, _value: *const [[f32; 2]; 2]) {}
	pub unsafe fn uniformMatrix3fv(_location: GLuint, _count: GLsizei, _transpose: GLboolean, _value: *const [[f32; 3]; 3]) {}
	pub unsafe fn uniformMatrix4fv(_location: GLuint, _count: GLsizei, _transpose: GLboolean, _value: *const [[f32; 4]; 4]) {}
	pub unsafe fn vertexAttribDivisor(_index: GLuint, _divisor: GLuint) {}

	pub unsafe fn createTexture() -> GLuint { 0 }
	pub unsafe fn deleteTexture(_texture: GLuint) {}
	pub unsafe fn activeTexture(_texture: GLenum) {}
	pub unsafe fn bindTexture(_target: GLenum, _texture: GLuint) {}
	pub unsafe fn generateMipmap(_target: GLenum) {}
	pub unsafe fn pixelStorei(_pname: GLenum, _param: GLint) {}
	pub unsafe fn texParameteri(_target: GLenum, _pname: GLenum, _param: GLint) {}
	pub unsafe fn texStorage2D(_target: GLenum, _levels: GLsizei, _internalformat: GLenum, _width: GLsizei, _height: GLsizei) {}
	pub unsafe fn texImage2D(_target: GLenum, _level: GLint, _internalformat: GLenum, _width: GLsizei, _height: GLsizei, _border: GLint, _format: GLenum, _type_: GLenum, _pixels_ptr: *const u8, _pixels_len: usize) {}
	pub unsafe fn texSubImage2D(_target: GLenum, _level: GLint, _xoffset: GLint, _yoffset: GLint, _width: GLsizei, _height: GLsizei, _format: GLenum, _type_: GLenum, _pixels_ptr: *const u8, _pixels_len: usize) {}

	pub unsafe fn createFramebuffer() -> GLuint { 0 }
	pub unsafe fn deleteFramebuffer(_framebuffer: GLuint) {}
	pub unsafe fn bindFramebuffer(_target: GLenum, _framebuffer: GLuint) {}
	pub unsafe fn framebufferTexture2D(_target: GLenum, _attachment: GLenum, _textarget: GLenum, _texture: GLuint, _level: GLint) {}
	pub unsafe fn drawBuffers(_n: GLsizei, _bufs: *const GLenum) {}
	pub unsafe fn readBuffer(_src: GLenum) {}
	pub unsafe fn checkFramebufferStatus(_target: GLenum) -> GLenum { 0 }
	pub unsafe fn readPixels(_x: GLint, _y: GLint, _width: GLsizei, _height: GLsizei, _format: GLenum, _type_: GLenum, _pixels_ptr: *mut u8, _pixels_len: usize) {}

	pub unsafe fn drawArrays(_mode: GLenum, _first: GLint, _count: GLsizei) {}
	pub unsafe fn drawElements(_mode: GLenum, _count: GLsizei, _type_: GLenum, _indices: GLintptr) {}
	pub unsafe fn drawArraysInstanced(_mode: GLenum, _first: GLint, _count: GLsizei, _instancecount: GLsizei) {}
	pub unsafe fn drawElementsInstanced(_mode: GLenum, _count: GLsizei, _type_: GLenum, _indices: GLintptr, _instancecount: GLsizei) {}
}

#[cfg(not(target_family = "wasm"))]
pub use stubs::*;

pub const FALSE: types::GLboolean = false;
pub const TRUE: types::GLboolean = true;

// Clearing buffers
pub const DEPTH_BUFFER_BIT: types::GLbitfield   = 0x00000100;
pub const STENCIL_BUFFER_BIT: types::GLbitfield = 0x00000400;
pub const COLOR_BUFFER_BIT: types::GLbitfield   = 0x00004000;

// Rendering primitives
pub const POINTS: types::GLenum = 0;
pub const LINES: types::GLenum = 1;
pub const LINE_LOOP: types::GLenum = 2;
pub const LINE_STRIP: types::GLenum = 3;
pub const TRIANGLES: types::GLenum = 4;
pub const TRIANGLE_STRIP: types::GLenum = 5;
pub const TRIANGLE_FAN: types::GLenum = 6;

// Blending modes
pub const ZERO: types::GLenum = 0;
pub const ONE: types::GLenum = 1;
pub const SRC_COLOR: types::GLenum = 0x0300;
pub const ONE_MINUS_SRC_COLOR: types::GLenum = 0x0301;
pub const SRC_ALPHA: types::GLenum = 0x0302;
pub const ONE_MINUS_SRC_ALPHA: types::GLenum = 0x0303;
pub const DST_ALPHA: types::GLenum = 0x0304;
pub const ONE_MINUS_DST_ALPHA: types::GLenum = 0x0305;
pub const DST_COLOR: types::GLenum = 0x0306;
pub const ONE_MINUS_DST_COLOR: types::GLenum = 0x0307;
pub const SRC_ALPHA_SATURATE: types::GLenum = 0x0308;
pub const CONSTANT_COLOR: types::GLenum = 0x8001;
pub const ONE_MINUS_CONSTANT_COLOR: types::GLenum = 0x8002;
pub const CONSTANT_ALPHA: types::GLenum = 0x8003;
pub const ONE_MINUS_CONSTANT_ALPHA: types::GLenum = 0x8004;

// Blending equations
pub const FUNC_ADD: types::GLenum = 0x8006;
pub const FUNC_SUBTRACT: types::GLenum = 0x800A;
pub const FUNC_REVERSE_SUBTRACT: types::GLenum = 0x800B;

// GL parameters
pub const BLEND_EQUATION: types::GLenum = 0x8009;
pub const BLEND_EQUATION_RGB: types::GLenum = 0x8009;
pub const BLEND_EQUATION_ALPHA: types::GLenum = 0x883D;
pub const BLEND_DST_RGB: types::GLenum = 0x80C8;
pub const BLEND_SRC_RGB: types::GLenum = 0x80C9;
pub const BLEND_DST_ALPHA: types::GLenum = 0x80CA;
pub const BLEND_SRC_ALPHA: types::GLenum = 0x80CB;
pub const BLEND_COLOR: types::GLenum = 0x8005;
pub const ARRAY_BUFFER_BINDING: types::GLenum = 0x8894;
pub const ELEMENT_ARRAY_BUFFER_BINDING: types::GLenum = 0x8895;
pub const LINE_WIDTH: types::GLenum = 0x0B21;
pub const ALIASED_POINT_SIZE_RANGE: types::GLenum = 0x846D;
pub const ALIASED_LINE_WIDTH_RANGE: types::GLenum = 0x846E;
pub const CULL_FACE_MODE: types::GLenum = 0x0B45;
pub const FRONT_FACE: types::GLenum = 0x0B46;
pub const DEPTH_RANGE: types::GLenum = 0x0B70;
pub const DEPTH_WRITEMASK: types::GLenum = 0x0B72;
pub const DEPTH_CLEAR_VALUE: types::GLenum = 0x0B73;
pub const DEPTH_FUNC: types::GLenum = 0x0B74;
pub const STENCIL_CLEAR_VALUE: types::GLenum = 0x0B91;
pub const STENCIL_FUNC: types::GLenum = 0x0B92;
pub const STENCIL_FAIL: types::GLenum = 0x0B94;
pub const STENCIL_PASS_DEPTH_FAIL: types::GLenum = 0x0B95;
pub const STENCIL_PASS_DEPTH_PASS: types::GLenum = 0x0B96;
pub const STENCIL_REF: types::GLenum = 0x0B97;
pub const STENCIL_VALUE_MASK: types::GLenum = 0x0B93;
pub const STENCIL_WRITEMASK: types::GLenum = 0x0B98;
pub const STENCIL_BACK_FUNC: types::GLenum = 0x8800;
pub const STENCIL_BACK_FAIL: types::GLenum = 0x8801;
pub const STENCIL_BACK_PASS_DEPTH_FAIL: types::GLenum = 0x8802;
pub const STENCIL_BACK_PASS_DEPTH_PASS: types::GLenum = 0x8803;
pub const STENCIL_BACK_REF: types::GLenum = 0x8CA3;
pub const STENCIL_BACK_VALUE_MASK: types::GLenum = 0x8CA4;
pub const STENCIL_BACK_WRITEMASK: types::GLenum = 0x8CA5;
pub const VIEWPORT: types::GLenum = 0x0BA2;
pub const SCISSOR_BOX: types::GLenum = 0x0C10;
pub const COLOR_CLEAR_VALUE: types::GLenum = 0x0C22;
pub const COLOR_WRITEMASK: types::GLenum = 0x0C23;
pub const UNPACK_ALIGNMENT: types::GLenum = 0x0CF5;
pub const PACK_ALIGNMENT: types::GLenum = 0x0D05;
pub const MAX_TEXTURE_SIZE: types::GLenum = 0x0D33;
pub const MAX_VIEWPORT_DIMS: types::GLenum = 0x0D3A;
pub const SUBPIXEL_BITS: types::GLenum = 0x0D50;
pub const RED_BITS: types::GLenum = 0x0D52;
pub const GREEN_BITS: types::GLenum = 0x0D53;
pub const BLUE_BITS: types::GLenum = 0x0D54;
pub const ALPHA_BITS: types::GLenum = 0x0D55;
pub const DEPTH_BITS: types::GLenum = 0x0D56;
pub const STENCIL_BITS: types::GLenum = 0x0D57;
pub const POLYGON_OFFSET_UNITS: types::GLenum = 0x2A00;
pub const POLYGON_OFFSET_FACTOR: types::GLenum = 0x8038;
pub const TEXTURE_BINDING_2D: types::GLenum = 0x8069;
pub const SAMPLE_BUFFERS: types::GLenum = 0x80A8;
pub const SAMPLES: types::GLenum = 0x80A9;
pub const SAMPLE_COVERAGE_VALUE: types::GLenum = 0x80AA;
pub const SAMPLE_COVERAGE_INVERT: types::GLenum = 0x80AB;
pub const COMPRESSED_TEXTURE_FORMATS: types::GLenum = 0x86A3;
pub const VENDOR: types::GLenum = 0x1F00;
pub const RENDERER: types::GLenum = 0x1F01;
pub const VERSION: types::GLenum = 0x1F02;

// Buffers
pub const STATIC_DRAW: types::GLenum = 0x88E4;
pub const STREAM_DRAW: types::GLenum = 0x88E0;
pub const DYNAMIC_DRAW: types::GLenum = 0x88E8;
pub const ARRAY_BUFFER: types::GLenum = 0x8892;
pub const ELEMENT_ARRAY_BUFFER: types::GLenum = 0x8893;
pub const BUFFER_SIZE: types::GLenum = 0x8764;
pub const BUFFER_USAGE: types::GLenum = 0x8765;

// Vertex attributes

// Culling
pub const CULL_FACE: types::GLenum = 0x0B44;
pub const FRONT: types::GLenum = 0x0404;
pub const BACK: types::GLenum = 0x0405;
pub const FRONT_AND_BACK: types::GLenum = 0x0408;

// Enabling and disabling
pub const BLEND: types::GLenum = 0x0BE2;
pub const DEPTH_TEST: types::GLenum = 0x0B71;
pub const DITHER: types::GLenum = 0x0BD0;
pub const POLYGON_OFFSET_FILL: types::GLenum = 0x8037;
pub const SAMPLE_ALPHA_TO_COVERAGE: types::GLenum = 0x809E;
pub const SAMPLE_COVERAGE: types::GLenum = 0x80A0;
pub const SCISSOR_TEST: types::GLenum = 0x0C11;
pub const STENCIL_TEST: types::GLenum = 0x0B90;

// Errors
pub const NO_ERROR: types::GLenum = 0;
pub const INVALID_ENUM: types::GLenum = 0x0500;
pub const INVALID_VALUE: types::GLenum = 0x0501;
pub const INVALID_OPERATION: types::GLenum = 0x0502;
pub const OUT_OF_MEMORY: types::GLenum = 0x0505;
pub const CONTEXT_LOST_WEBGL: types::GLenum = 0x9242;

// Front face directions
pub const CW: types::GLenum = 0x0900;
pub const CCW: types::GLenum = 0x0901;

// Hints

// Data types
pub const BYTE: types::GLenum = 0x1400;
pub const UNSIGNED_BYTE: types::GLenum = 0x1401;
pub const SHORT: types::GLenum = 0x1402;
pub const UNSIGNED_SHORT: types::GLenum = 0x1403;
pub const INT: types::GLenum = 0x1404;
pub const UNSIGNED_INT: types::GLenum = 0x1405;
pub const FLOAT: types::GLenum = 0x1406;

// Pixel formats
pub const DEPTH_COMPONENT: types::GLenum = 0x1902;
pub const ALPHA: types::GLenum = 0x1906;
pub const RGB: types::GLenum = 0x1907;
pub const RGBA: types::GLenum = 0x1908;
pub const LUMINANCE: types::GLenum = 0x1909;
pub const LUMINANCE_ALPHA: types::GLenum = 0x190A;

// Pixel types
pub const UNSIGNED_SHORT_4_4_4_4: types::GLenum = 0x8033;
pub const UNSIGNED_SHORT_5_5_5_1: types::GLenum = 0x8034;
pub const UNSIGNED_SHORT_5_6_5: types::GLenum = 0x8363;

// Shaders
pub const FRAGMENT_SHADER: types::GLenum = 0x8B30;
pub const VERTEX_SHADER: types::GLenum = 0x8B31;
pub const COMPILE_STATUS: types::GLenum = 0x8B81;
pub const DELETE_STATUS: types::GLenum = 0x8B80;
pub const LINK_STATUS: types::GLenum = 0x8B82;
pub const VALIDATE_STATUS: types::GLenum = 0x8B83;
pub const ATTACHED_SHADERS: types::GLenum = 0x8B85;
pub const ACTIVE_ATTRIBUTES: types::GLenum = 0x8B89;
pub const ACTIVE_UNIFORMS: types::GLenum = 0x8B86;
pub const MAX_VERTEX_ATTRIBS: types::GLenum = 0x8869;
pub const MAX_VERTEX_UNIFORM_VECTORS: types::GLenum = 0x8DFB;
pub const MAX_VARYING_VECTORS: types::GLenum = 0x8DFC;
pub const MAX_COMBINED_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8B4D;
pub const MAX_VERTEX_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8B4C;
pub const MAX_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8872;
pub const MAX_FRAGMENT_UNIFORM_VECTORS: types::GLenum = 0x8DFD;
pub const SHADER_TYPE: types::GLenum = 0x8B4F;
pub const SHADING_LANGUAGE_VERSION: types::GLenum = 0x8B8C;
pub const CURRENT_PROGRAM: types::GLenum = 0x8B8D;

// Depth or stencil tests
pub const NEVER: types::GLenum = 0x0200;
pub const LESS: types::GLenum = 0x0201;
pub const EQUAL: types::GLenum = 0x0202;
pub const LEQUAL: types::GLenum = 0x0203;
pub const GREATER: types::GLenum = 0x0204;
pub const NOTEQUAL: types::GLenum = 0x0205;
pub const GEQUAL: types::GLenum = 0x0206;
pub const ALWAYS: types::GLenum = 0x0207;

// Stencil actions
pub const KEEP: types::GLenum = 0x1E00;
pub const REPLACE: types::GLenum = 0x1E01;
pub const INCR: types::GLenum = 0x1E02;
pub const DECR: types::GLenum = 0x1E03;
pub const INVERT: types::GLenum = 0x150A;
pub const INCR_WRAP: types::GLenum = 0x8507;
pub const DECR_WRAP: types::GLenum = 0x8508;

// Textures
pub const NEAREST: types::GLenum = 0x2600;
pub const LINEAR: types::GLenum = 0x2601;
pub const NEAREST_MIPMAP_NEAREST: types::GLenum = 0x2700;
pub const LINEAR_MIPMAP_NEAREST: types::GLenum = 0x2701;
pub const NEAREST_MIPMAP_LINEAR: types::GLenum = 0x2702;
pub const LINEAR_MIPMAP_LINEAR: types::GLenum = 0x2703;
pub const TEXTURE_MAG_FILTER: types::GLenum = 0x2800;
pub const TEXTURE_MIN_FILTER: types::GLenum = 0x2801;
pub const TEXTURE_WRAP_S: types::GLenum = 0x2802;
pub const TEXTURE_WRAP_T: types::GLenum = 0x2803;
pub const TEXTURE_2D: types::GLenum = 0x0DE1;
pub const TEXTURE: types::GLenum = 0x1702;
pub const TEXTURE_CUBE_MAP: types::GLenum = 0x8513;
pub const TEXTURE_BINDING_CUBE_MAP: types::GLenum = 0x8514;
pub const TEXTURE_CUBE_MAP_POSITIVE_X: types::GLenum = 0x8515;
pub const TEXTURE_CUBE_MAP_NEGATIVE_X: types::GLenum = 0x8516;
pub const TEXTURE_CUBE_MAP_POSITIVE_Y: types::GLenum = 0x8517;
pub const TEXTURE_CUBE_MAP_NEGATIVE_Y: types::GLenum = 0x8518;
pub const TEXTURE_CUBE_MAP_POSITIVE_Z: types::GLenum = 0x8519;
pub const TEXTURE_CUBE_MAP_NEGATIVE_Z: types::GLenum = 0x851A;
pub const MAX_CUBE_MAP_TEXTURE_SIZE: types::GLenum = 0x851C;
pub const TEXTURE0: types::GLenum = 0x84C0; // Up to +31 texture units
pub const ACTIVE_TEXTURE: types::GLenum = 0x84E0;
pub const REPEAT: types::GLenum = 0x2901;
pub const CLAMP_TO_EDGE: types::GLenum = 0x812F;
pub const MIRRORED_REPEAT: types::GLenum = 0x8370;

// Uniform types
pub const FLOAT_VEC2: types::GLenum = 0x8B50;
pub const FLOAT_VEC3: types::GLenum = 0x8B51;
pub const FLOAT_VEC4: types::GLenum = 0x8B52;
pub const INT_VEC2: types::GLenum = 0x8B53;
pub const INT_VEC3: types::GLenum = 0x8B54;
pub const INT_VEC4: types::GLenum = 0x8B55;
pub const BOOL: types::GLenum = 0x8B56;
pub const BOOL_VEC2: types::GLenum = 0x8B57;
pub const BOOL_VEC3: types::GLenum = 0x8B58;
pub const BOOL_VEC4: types::GLenum = 0x8B59;
pub const FLOAT_MAT2: types::GLenum = 0x8B5A;
pub const FLOAT_MAT3: types::GLenum = 0x8B5B;
pub const FLOAT_MAT4: types::GLenum = 0x8B5C;
pub const SAMPLER_2D: types::GLenum = 0x8B5E;
pub const SAMPLER_CUBE: types::GLenum = 0x8B60;

// Shader precision-specified types
pub const LOW_FLOAT: types::GLenum = 0x8DF0;
pub const MEDIUM_FLOAT: types::GLenum = 0x8DF1;
pub const HIGH_FLOAT: types::GLenum = 0x8DF2;
pub const LOW_INT: types::GLenum = 0x8DF3;
pub const MEDIUM_INT: types::GLenum = 0x8DF4;
pub const HIGH_INT: types::GLenum = 0x8DF5;

// Framebuffers and renderbuffers
pub const FRAMEBUFFER: types::GLenum = 0x8D40;
pub const RENDERBUFFER: types::GLenum = 0x8D41;
pub const RGBA4: types::GLenum = 0x8056;
pub const RGB5_A1: types::GLenum = 0x8057;
pub const DEPTH_COMPONENT16: types::GLenum = 0x81A5;
pub const STENCIL_INDEX8: types::GLenum = 0x8D48;
pub const RENDERBUFFER_WIDTH: types::GLenum = 0x8D42;
pub const RENDERBUFFER_HEIGHT: types::GLenum = 0x8D43;
pub const RENDERBUFFER_INTERNAL_FORMAT: types::GLenum = 0x8D44;
pub const RENDERBUFFER_RED_SIZE: types::GLenum = 0x8D50;
pub const RENDERBUFFER_GREEN_SIZE: types::GLenum = 0x8D51;
pub const RENDERBUFFER_BLUE_SIZE: types::GLenum = 0x8D52;
pub const RENDERBUFFER_ALPHA_SIZE: types::GLenum = 0x8D53;
pub const RENDERBUFFER_DEPTH_SIZE: types::GLenum = 0x8D54;
pub const FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: types::GLenum = 0x8CD0;
pub const FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: types::GLenum = 0x8CD1;
pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: types::GLenum = 0x8CD2;
pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: types::GLenum = 0x8CD3;
pub const COLOR_ATTACHMENT0: types::GLenum = 0x8CE0;
pub const DEPTH_ATTACHMENT: types::GLenum = 0x8D00;
pub const STENCIL_ATTACHMENT: types::GLenum = 0x8D20;
pub const NONE: types::GLenum = 0;
pub const FRAMEBUFFER_COMPLETE: types::GLenum = 0x8CD5;
pub const FRAMEBUFFER_INCOMPLETE_ATTACHMENT: types::GLenum = 0x8CD6;
pub const FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: types::GLenum = 0x8CD7;
pub const FRAMEBUFFER_INCOMPLETE_DIMENSIONS: types::GLenum = 0x8CD9;
pub const FRAMEBUFFER_UNSUPPORTED: types::GLenum = 0x8CDD;
pub const FRAMEBUFFER_BINDING: types::GLenum = 0x8CA6;
pub const RENDERBUFFER_BINDING: types::GLenum = 0x8CA7;
pub const MAX_RENDERBUFFER_SIZE: types::GLenum = 0x84E8;
pub const INVALID_FRAMEBUFFER_OPERATION: types::GLenum = 0x0506;

// Pixel storage modes
pub const UNPACK_FLIP_Y_WEBGL: types::GLenum = 0x9240;
pub const UNPACK_PREMULTIPLY_ALPHA_WEBGL: types::GLenum = 0x9241;
pub const UNPACK_COLORSPACE_CONVERSION_WEBGL: types::GLenum = 0x9243;

//----------------------------------------------------------------
// Additional constants defined WebGL 2

// Getting GL parameter information
pub const READ_BUFFER: types::GLenum = 0x0C02;
pub const UNPACK_ROW_LENGTH: types::GLenum = 0x0CF2;
pub const UNPACK_SKIP_ROWS: types::GLenum = 0x0CF3;
pub const UNPACK_SKIP_PIXELS: types::GLenum = 0x0CF4;
pub const PACK_ROW_LENGTH: types::GLenum = 0x0D02;
pub const PACK_SKIP_ROWS: types::GLenum = 0x0D03;
pub const PACK_SKIP_PIXELS: types::GLenum = 0x0D04;
pub const TEXTURE_BINDING_3D: types::GLenum = 0x806A;
pub const UNPACK_SKIP_IMAGES: types::GLenum = 0x806D;
pub const UNPACK_IMAGE_HEIGHT: types::GLenum = 0x806E;
pub const MAX_3D_TEXTURE_SIZE: types::GLenum = 0x8073;
pub const MAX_ELEMENTS_VERTICES: types::GLenum = 0x80E8;
pub const MAX_ELEMENTS_INDICES: types::GLenum = 0x80E9;
pub const MAX_TEXTURE_LOD_BIAS: types::GLenum = 0x84FD;
pub const MAX_FRAGMENT_UNIFORM_COMPONENTS: types::GLenum = 0x8B49;
pub const MAX_VERTEX_UNIFORM_COMPONENTS: types::GLenum = 0x8B4A;
pub const MAX_ARRAY_TEXTURE_LAYERS: types::GLenum = 0x88FF;
pub const MIN_PROGRAM_TEXEL_OFFSET: types::GLenum = 0x8904;
pub const MAX_PROGRAM_TEXEL_OFFSET: types::GLenum = 0x8905;
pub const MAX_VARYING_COMPONENTS: types::GLenum = 0x8B4B;
pub const FRAGMENT_SHADER_DERIVATIVE_HINT: types::GLenum = 0x8B8B;
pub const RASTERIZER_DISCARD: types::GLenum = 0x8C89;
pub const VERTEX_ARRAY_BINDING: types::GLenum = 0x85B5;
pub const MAX_VERTEX_OUTPUT_COMPONENTS: types::GLenum = 0x9122;
pub const MAX_FRAGMENT_INPUT_COMPONENTS: types::GLenum = 0x9125;
pub const MAX_SERVER_WAIT_TIMEOUT: types::GLenum = 0x9111;
pub const MAX_ELEMENT_INDEX: types::GLenum = 0x8D6B;

// Textures
pub const RED: types::GLenum = 0x1903;
pub const RGB8: types::GLenum = 0x8051;
pub const RGBA8: types::GLenum = 0x8058;
pub const RGB10_A2: types::GLenum = 0x8059;
pub const TEXTURE_3D: types::GLenum = 0x806F;
pub const TEXTURE_WRAP_R: types::GLenum = 0x8072;
pub const TEXTURE_MIN_LOD: types::GLenum = 0x813A;
pub const TEXTURE_MAX_LOD: types::GLenum = 0x813B;
pub const TEXTURE_BASE_LEVEL: types::GLenum = 0x813C;
pub const TEXTURE_MAX_LEVEL: types::GLenum = 0x813D;
pub const TEXTURE_COMPARE_MODE: types::GLenum = 0x884C;
pub const TEXTURE_COMPARE_FUNC: types::GLenum = 0x884D;
pub const SRGB: types::GLenum = 0x8C40;
pub const SRGB8: types::GLenum = 0x8C41;
pub const SRGB8_ALPHA8: types::GLenum = 0x8C43;
pub const COMPARE_REF_TO_TEXTURE: types::GLenum = 0x884E;
pub const RGBA32F: types::GLenum = 0x8814;
pub const RGB32F: types::GLenum = 0x8815;
pub const RGBA16F: types::GLenum = 0x881A;
pub const RGB16F: types::GLenum = 0x881B;
pub const TEXTURE_2D_ARRAY: types::GLenum = 0x8C1A;
pub const TEXTURE_BINDING_2D_ARRAY: types::GLenum = 0x8C1D;
pub const R11F_G11F_B10F: types::GLenum = 0x8C3A;
pub const RGB9_E5: types::GLenum = 0x8C3D;
pub const RGBA32UI: types::GLenum = 0x8D70;
pub const RGB32UI: types::GLenum = 0x8D71;
pub const RGBA16UI: types::GLenum = 0x8D76;
pub const RGB16UI: types::GLenum = 0x8D77;
pub const RGBA8UI: types::GLenum = 0x8D7C;
pub const RGB8UI: types::GLenum = 0x8D7D;
pub const RGBA32I: types::GLenum = 0x8D82;
pub const RGB32I: types::GLenum = 0x8D83;
pub const RGBA16I: types::GLenum = 0x8D88;
pub const RGB16I: types::GLenum = 0x8D89;
pub const RGBA8I: types::GLenum = 0x8D8E;
pub const RGB8I: types::GLenum = 0x8D8F;
pub const RED_INTEGER: types::GLenum = 0x8D94;
pub const RGB_INTEGER: types::GLenum = 0x8D98;
pub const RGBA_INTEGER: types::GLenum = 0x8D99;
pub const R8: types::GLenum = 0x8229;
pub const RG8: types::GLenum = 0x822B;
pub const R16F: types::GLenum = 0x822D;
pub const R32F: types::GLenum = 0x822E;
pub const RG16F: types::GLenum = 0x822F;
pub const RG32F: types::GLenum = 0x8230;
pub const R8I: types::GLenum = 0x8231;
pub const R8UI: types::GLenum = 0x8232;
pub const R16I: types::GLenum = 0x8233;
pub const R16UI: types::GLenum = 0x8234;
pub const R32I: types::GLenum = 0x8235;
pub const R32UI: types::GLenum = 0x8236;
pub const RG8I: types::GLenum = 0x8237;
pub const RG8UI: types::GLenum = 0x8238;
pub const RG16I: types::GLenum = 0x8239;
pub const RG16UI: types::GLenum = 0x823A;
pub const RG32I: types::GLenum = 0x823B;
pub const RG32UI: types::GLenum = 0x823C;
pub const R8_SNORM: types::GLenum = 0x8F94;
pub const RG8_SNORM: types::GLenum = 0x8F95;
pub const RGB8_SNORM: types::GLenum = 0x8F96;
pub const RGBA8_SNORM: types::GLenum = 0x8F97;
pub const RGB10_A2UI: types::GLenum = 0x906F;
pub const TEXTURE_IMMUTABLE_FORMAT: types::GLenum = 0x912F;
pub const TEXTURE_IMMUTABLE_LEVELS: types::GLenum = 0x82DF;

// Pixel types
pub const UNSIGNED_INT_2_10_10_10_REV: types::GLenum = 0x8368;
pub const UNSIGNED_INT_10F_11F_11F_REV: types::GLenum = 0x8C3B;
pub const UNSIGNED_INT_5_9_9_9_REV: types::GLenum = 0x8C3E;
pub const FLOAT_32_UNSIGNED_INT_24_8_REV: types::GLenum = 0x8DAD;
pub const UNSIGNED_INT_24_8: types::GLenum = 0x84FA;
pub const HALF_FLOAT: types::GLenum = 0x140B;
pub const RG: types::GLenum = 0x8227;
pub const RG_INTEGER: types::GLenum = 0x8228;
pub const INT_2_10_10_10_REV: types::GLenum = 0x8D9F;

// Queries
pub const CURRENT_QUERY: types::GLenum = 0x8865;
pub const QUERY_RESULT: types::GLenum = 0x8866;
pub const QUERY_RESULT_AVAILABLE: types::GLenum = 0x8867;
pub const ANY_SAMPLES_PASSED: types::GLenum = 0x8C2F;
pub const ANY_SAMPLES_PASSED_CONSERVATIVE: types::GLenum = 0x8D6A;

// Draw buffers
pub const MAX_DRAW_BUFFERS: types::GLenum = 0x8824;
pub const DRAW_BUFFER0: types::GLenum = 0x8825;
pub const DRAW_BUFFER1: types::GLenum = 0x8826;
pub const DRAW_BUFFER2: types::GLenum = 0x8827;
pub const DRAW_BUFFER3: types::GLenum = 0x8828;
pub const DRAW_BUFFER4: types::GLenum = 0x8829;
pub const DRAW_BUFFER5: types::GLenum = 0x882A;
pub const DRAW_BUFFER6: types::GLenum = 0x882B;
pub const DRAW_BUFFER7: types::GLenum = 0x882C;
pub const DRAW_BUFFER8: types::GLenum = 0x882D;
pub const DRAW_BUFFER9: types::GLenum = 0x882E;
pub const DRAW_BUFFER10: types::GLenum = 0x882F;
pub const DRAW_BUFFER11: types::GLenum = 0x8830;
pub const DRAW_BUFFER12: types::GLenum = 0x8831;
pub const DRAW_BUFFER13: types::GLenum = 0x8832;
pub const DRAW_BUFFER14: types::GLenum = 0x8833;
pub const DRAW_BUFFER15: types::GLenum = 0x8834;
pub const MAX_COLOR_ATTACHMENTS: types::GLenum = 0x8CDF;
pub const COLOR_ATTACHMENT1: types::GLenum = 0x8CE1;
pub const COLOR_ATTACHMENT2: types::GLenum = 0x8CE2;
pub const COLOR_ATTACHMENT3: types::GLenum = 0x8CE3;
pub const COLOR_ATTACHMENT4: types::GLenum = 0x8CE4;
pub const COLOR_ATTACHMENT5: types::GLenum = 0x8CE5;
pub const COLOR_ATTACHMENT6: types::GLenum = 0x8CE6;
pub const COLOR_ATTACHMENT7: types::GLenum = 0x8CE7;
pub const COLOR_ATTACHMENT8: types::GLenum = 0x8CE8;
pub const COLOR_ATTACHMENT9: types::GLenum = 0x8CE9;
pub const COLOR_ATTACHMENT10: types::GLenum = 0x8CEA;
pub const COLOR_ATTACHMENT11: types::GLenum = 0x8CEB;
pub const COLOR_ATTACHMENT12: types::GLenum = 0x8CEC;
pub const COLOR_ATTACHMENT13: types::GLenum = 0x8CED;
pub const COLOR_ATTACHMENT14: types::GLenum = 0x8CEE;
pub const COLOR_ATTACHMENT15: types::GLenum = 0x8CEF;

// Samplers
pub const SAMPLER_3D: types::GLenum = 0x8B5F;
pub const SAMPLER_2D_SHADOW: types::GLenum = 0x8B62;
pub const SAMPLER_2D_ARRAY: types::GLenum = 0x8DC1;
pub const SAMPLER_2D_ARRAY_SHADOW: types::GLenum = 0x8DC4;
pub const SAMPLER_CUBE_SHADOW: types::GLenum = 0x8DC5;
pub const INT_SAMPLER_2D: types::GLenum = 0x8DCA;
pub const INT_SAMPLER_3D: types::GLenum = 0x8DCB;
pub const INT_SAMPLER_CUBE: types::GLenum = 0x8DCC;
pub const INT_SAMPLER_2D_ARRAY: types::GLenum = 0x8DCF;
pub const UNSIGNED_INT_SAMPLER_2D: types::GLenum = 0x8DD2;
pub const UNSIGNED_INT_SAMPLER_3D: types::GLenum = 0x8DD3;
pub const UNSIGNED_INT_SAMPLER_CUBE: types::GLenum = 0x8DD4;
pub const UNSIGNED_INT_SAMPLER_2D_ARRAY: types::GLenum = 0x8DD7;
pub const MAX_SAMPLES: types::GLenum = 0x8D57;
pub const SAMPLER_BINDING: types::GLenum = 0x8919;

// Buffers
pub const PIXEL_PACK_BUFFER: types::GLenum = 0x88EB;
pub const PIXEL_UNPACK_BUFFER: types::GLenum = 0x88EC;
pub const PIXEL_PACK_BUFFER_BINDING: types::GLenum = 0x88ED;
pub const PIXEL_UNPACK_BUFFER_BINDING: types::GLenum = 0x88EF;
pub const COPY_READ_BUFFER: types::GLenum = 0x8F36;
pub const COPY_WRITE_BUFFER: types::GLenum = 0x8F37;
pub const COPY_READ_BUFFER_BINDING: types::GLenum = 0x8F36;
pub const COPY_WRITE_BUFFER_BINDING: types::GLenum = 0x8F37;

// Data types
pub const FLOAT_MAT2x3: types::GLenum = 0x8B65;
pub const FLOAT_MAT2x4: types::GLenum = 0x8B66;
pub const FLOAT_MAT3x2: types::GLenum = 0x8B67;
pub const FLOAT_MAT3x4: types::GLenum = 0x8B68;
pub const FLOAT_MAT4x2: types::GLenum = 0x8B69;
pub const FLOAT_MAT4x3: types::GLenum = 0x8B6A;
pub const UNSIGNED_INT_VEC2: types::GLenum = 0x8DC6;
pub const UNSIGNED_INT_VEC3: types::GLenum = 0x8DC7;
pub const UNSIGNED_INT_VEC4: types::GLenum = 0x8DC8;
pub const UNSIGNED_NORMALIZED: types::GLenum = 0x8C17;
pub const SIGNED_NORMALIZED: types::GLenum = 0x8F9C;

// Vertex attributes
pub const VERTEX_ATTRIB_ARRAY_INTEGER: types::GLenum = 0x88FD;
pub const VERTEX_ATTRIB_ARRAY_DIVISOR: types::GLenum = 0x88FE;

// Transform feedback
pub const TRANSFORM_FEEDBACK_BUFFER_MODE: types::GLenum = 0x8C7F;
pub const MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS: types::GLenum = 0x8C80;
pub const TRANSFORM_FEEDBACK_VARYINGS: types::GLenum = 0x8C83;
pub const TRANSFORM_FEEDBACK_BUFFER_START: types::GLenum = 0x8C84;
pub const TRANSFORM_FEEDBACK_BUFFER_SIZE: types::GLenum = 0x8C85;
pub const TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN: types::GLenum = 0x8C88;
pub const MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS: types::GLenum = 0x8C8A;
pub const MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS: types::GLenum = 0x8C8B;
pub const INTERLEAVED_ATTRIBS: types::GLenum = 0x8C8C;
pub const SEPARATE_ATTRIBS: types::GLenum = 0x8C8D;
pub const TRANSFORM_FEEDBACK_BUFFER: types::GLenum = 0x8C8E;
pub const TRANSFORM_FEEDBACK_BUFFER_BINDING: types::GLenum = 0x8C8F;
pub const TRANSFORM_FEEDBACK: types::GLenum = 0x8E22;
pub const TRANSFORM_FEEDBACK_PAUSED: types::GLenum = 0x8E23;
pub const TRANSFORM_FEEDBACK_ACTIVE: types::GLenum = 0x8E24;
pub const TRANSFORM_FEEDBACK_BINDING: types::GLenum = 0x8E25;

// Framebuffers and renderbuffers
pub const FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING: types::GLenum = 0x8210;
pub const FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE: types::GLenum = 0x8211;
pub const FRAMEBUFFER_ATTACHMENT_RED_SIZE: types::GLenum = 0x8212;
pub const FRAMEBUFFER_ATTACHMENT_GREEN_SIZE: types::GLenum = 0x8213;
pub const FRAMEBUFFER_ATTACHMENT_BLUE_SIZE: types::GLenum = 0x8214;
pub const FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE: types::GLenum = 0x8215;
pub const FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE: types::GLenum = 0x8216;
pub const FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE: types::GLenum = 0x8217;
pub const FRAMEBUFFER_DEFAULT: types::GLenum = 0x8218;
pub const DEPTH_STENCIL_ATTACHMENT: types::GLenum = 0x821A;
pub const DEPTH_STENCIL: types::GLenum = 0x84F9;
pub const DEPTH24_STENCIL8: types::GLenum = 0x88F0;
pub const DRAW_FRAMEBUFFER_BINDING: types::GLenum = 0x8CA6;
pub const READ_FRAMEBUFFER: types::GLenum = 0x8CA8;
pub const DRAW_FRAMEBUFFER: types::GLenum = 0x8CA9;
pub const READ_FRAMEBUFFER_BINDING: types::GLenum = 0x8CAA;
pub const RENDERBUFFER_SAMPLES: types::GLenum = 0x8CAB;
pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER: types::GLenum = 0x8CD4;
pub const FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: types::GLenum = 0x8D56;

// Uniforms
pub const UNIFORM_BUFFER: types::GLenum = 0x8A11;
pub const UNIFORM_BUFFER_BINDING: types::GLenum = 0x8A28;
pub const UNIFORM_BUFFER_START: types::GLenum = 0x8A29;
pub const UNIFORM_BUFFER_SIZE: types::GLenum = 0x8A2A;
pub const MAX_VERTEX_UNIFORM_BLOCKS: types::GLenum = 0x8A2B;
pub const MAX_FRAGMENT_UNIFORM_BLOCKS: types::GLenum = 0x8A2D;
pub const MAX_COMBINED_UNIFORM_BLOCKS: types::GLenum = 0x8A2E;
pub const MAX_UNIFORM_BUFFER_BINDINGS: types::GLenum = 0x8A2F;
pub const MAX_UNIFORM_BLOCK_SIZE: types::GLenum = 0x8A30;
pub const MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS: types::GLenum = 0x8A31;
pub const MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS: types::GLenum = 0x8A33;
pub const UNIFORM_BUFFER_OFFSET_ALIGNMENT: types::GLenum = 0x8A34;
pub const ACTIVE_UNIFORM_BLOCKS: types::GLenum = 0x8A36;
pub const UNIFORM_TYPE: types::GLenum = 0x8A37;
pub const UNIFORM_SIZE: types::GLenum = 0x8A38;
pub const UNIFORM_BLOCK_INDEX: types::GLenum = 0x8A3A;
pub const UNIFORM_OFFSET: types::GLenum = 0x8A3B;
pub const UNIFORM_ARRAY_STRIDE: types::GLenum = 0x8A3C;
pub const UNIFORM_MATRIX_STRIDE: types::GLenum = 0x8A3D;
pub const UNIFORM_IS_ROW_MAJOR: types::GLenum = 0x8A3E;
pub const UNIFORM_BLOCK_BINDING: types::GLenum = 0x8A3F;
pub const UNIFORM_BLOCK_DATA_SIZE: types::GLenum = 0x8A40;
pub const UNIFORM_BLOCK_ACTIVE_UNIFORMS: types::GLenum = 0x8A42;
pub const UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES: types::GLenum = 0x8A43;
pub const UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER: types::GLenum = 0x8A44;
pub const UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER: types::GLenum = 0x8A46;

// Sync objects
pub const OBJECT_TYPE: types::GLenum = 0x9112;
pub const SYNC_CONDITION: types::GLenum = 0x9113;
pub const SYNC_STATUS: types::GLenum = 0x9114;
pub const SYNC_FLAGS: types::GLenum = 0x9115;
pub const SYNC_FENCE: types::GLenum = 0x9116;
pub const SYNC_GPU_COMMANDS_COMPLETE: types::GLenum = 0x9117;
pub const UNSIGNALED: types::GLenum = 0x9118;
pub const SIGNALED: types::GLenum = 0x9119;
pub const ALREADY_SIGNALED: types::GLenum = 0x911A;
pub const TIMEOUT_EXPIRED: types::GLenum = 0x911B;
pub const CONDITION_SATISFIED: types::GLenum = 0x911C;
pub const WAIT_FAILED: types::GLenum = 0x911D;
pub const SYNC_FLUSH_COMMANDS_BIT: types::GLenum = 0x00000001;

// Miscellaneous constants
pub const COLOR: types::GLenum = 0x1800;
pub const DEPTH: types::GLenum = 0x1801;
pub const STENCIL: types::GLenum = 0x1802;
pub const MIN: types::GLenum = 0x8007;
pub const MAX: types::GLenum = 0x8008;
pub const DEPTH_COMPONENT24: types::GLenum = 0x81A6;
pub const STREAM_READ: types::GLenum = 0x88E1;
pub const STREAM_COPY: types::GLenum = 0x88E2;
pub const STATIC_READ: types::GLenum = 0x88E5;
pub const STATIC_COPY: types::GLenum = 0x88E6;
pub const DYNAMIC_READ: types::GLenum = 0x88E9;
pub const DYNAMIC_COPY: types::GLenum = 0x88EA;
pub const DEPTH_COMPONENT32F: types::GLenum = 0x8CAC;
pub const DEPTH32F_STENCIL8: types::GLenum = 0x8CAD;
pub const INVALID_INDEX: types::GLenum = 0xFFFFFFFF;
pub const TIMEOUT_IGNORED: types::GLenum = !0;
pub const MAX_CLIENT_WAIT_TIMEOUT_WEBGL: types::GLenum = 0x9247;
