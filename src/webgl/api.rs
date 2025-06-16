#![allow(dead_code)]

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
	pub fn colorMask(red: types::GLboolean, green: types::GLboolean, blue: types::GLboolean, alpha: types::GLboolean);
	pub fn depthMask(flag: types::GLboolean);
	pub fn stencilMask(mask: types::GLuint);
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
	pub fn getUniformLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	pub fn getAttribLocation(program: types::GLuint, name_ptr: *const u8, name_len: usize) -> types::GLint;
	pub fn uniform1fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform2fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform3fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform4fv(location: types::GLint, count: types::GLsizei, value: *const f32);
	pub fn uniform1iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform2iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform3iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniform4iv(location: types::GLint, count: types::GLsizei, value: *const types::GLint);
	pub fn uniformMatrix2fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix3fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn uniformMatrix4fv(location: types::GLint, count: types::GLsizei, transpose: types::GLboolean, value: *const f32);
	pub fn createTexture() -> types::GLuint;
	pub fn deleteTexture(texture: types::GLuint);
	pub fn activeTexture(texture: types::GLenum);
	pub fn bindTexture(target: types::GLenum, texture: types::GLuint);
	pub fn pixelStorei(pname: types::GLenum, param: types::GLint);
	pub fn texParameteri(target: types::GLenum, pname: types::GLenum, param: types::GLint);
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
	pub fn drawArrays(mode: types::GLenum, first: types::GLint, count: types::GLsizei);
	pub fn drawElements(mode: types::GLenum, count: types::GLsizei, type_: types::GLenum, indices: types::GLintptr);
}

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
