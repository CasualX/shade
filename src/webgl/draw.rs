use super::*;

fn gl_draw_mask(mask: &crate::DrawMask) {
	let red = if mask.red { api::TRUE } else { api::FALSE };
	let green = if mask.green { api::TRUE } else { api::FALSE };
	let blue = if mask.blue { api::TRUE } else { api::FALSE };
	let alpha = if mask.alpha { api::TRUE } else { api::FALSE };
	unsafe { api::colorMask(red, green, blue, alpha) };
	let depth = if mask.depth { api::TRUE } else { api::FALSE };
	unsafe { api::depthMask(depth) };
}

fn gl_blend(blend_mode: crate::BlendMode) {
	struct GlBlend {
		sfactor: api::types::GLenum,
		dfactor: api::types::GLenum,
		equation: api::types::GLenum,
	}
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
		crate::BlendMode::Lighten => unimplemented!("Lighten blend mode is not supported in WebGL 1.0"),
		crate::BlendMode::Screen => GlBlend {
			sfactor: api::ONE,
			dfactor: api::ONE_MINUS_SRC_COLOR,
			equation: api::FUNC_ADD,
		},
		crate::BlendMode::Darken => unimplemented!("Darken blend mode is not supported in WebGL 1.0"),
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

fn gl_scissor(scissor: &Option<cvmath::Bounds2<i32>>) {
	if let Some(scissor) = scissor {
		unsafe { api::enable(api::SCISSOR_TEST) };
		unsafe { api::scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()) };
	}
	else {
		unsafe { api::disable(api::SCISSOR_TEST) };
	}
}

fn gl_viewport(viewport: &cvmath::Bounds2<i32>) {
	unsafe { api::viewport(viewport.mins.x, viewport.mins.y, viewport.width(), viewport.height()) };
}

fn gl_attributes(shader: &WebGLProgram, data: &[crate::DrawVertexBuffer], map: &ResourceMap<WebGLVertexBuffer>) -> u32 {
	let mut enabled_attribs = 0u32;
	for vb in data {
		let Some(buf) = map.get(vb.buffer) else { continue }; // Validated in draw calls
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };

		let layout = buf.layout;
		for attr in layout.attributes {
			let Some(attrib) = shader.get_attrib(attr.name) else { continue };
			enabled_attribs |= 1 << attrib.location;
			unsafe { api::enableVertexAttribArray(attrib.location) };

			let size = attr.format.size() as GLint;
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
			unsafe { api::vertexAttribPointer(attrib.location, size, type_, normalized, layout.size as GLsizei, attr.offset as GLintptr) };
		}
	}
	unsafe { api::bindBuffer(api::ARRAY_BUFFER, 0) };

	// Assert that all attributes are bound
	#[cfg(debug_assertions)]
	for attr in &shader.attribs {
		assert!(enabled_attribs & (1 << attr.location) != 0, "Attribute {} not bound in shader {}", attr.name(), shader.program);
	}
	return enabled_attribs;
}

fn gl_attributes_disable(enabled_attribs: u32) {
	for i in 0..32 {
		if enabled_attribs & (1 << i) != 0 {
			unsafe { api::disableVertexAttribArray(i) };
		}
	}
}

struct WebGLUniformSetter<'a> {
	shader: &'a WebGLProgram,
	textures: &'a WebGLTextures,
}
impl<'a> crate::UniformSetter for WebGLUniformSetter<'a> {
	fn float(&mut self, name: &str, data: &[f32]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT, "Uniform {name:?} expected `float` type in shader");
			unsafe { api::uniform1fv(u.location, data.len() as i32, data.as_ptr()) };
		}
	}
	fn vec2(&mut self, name: &str, data: &[cvmath::Vec2<f32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_VEC2, "Uniform {name:?} expected `vec2` type in shader");
			unsafe { api::uniform2fv(u.location, data.len() as i32, data.as_ptr() as *const [f32; 2]) };
		}
	}
	fn vec3(&mut self, name: &str, data: &[cvmath::Vec3<f32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_VEC3, "Uniform {name:?} expected `vec3` type in shader");
			unsafe { api::uniform3fv(u.location, data.len() as i32, data.as_ptr() as *const [f32; 3]) };
		}
	}
	fn vec4(&mut self, name: &str, data: &[cvmath::Vec4<f32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_VEC4, "Uniform {name:?} expected `vec4` type in shader");
			unsafe { api::uniform4fv(u.location, data.len() as i32, data.as_ptr() as *const [f32; 4]) };
		}
	}
	fn int(&mut self, name: &str, data: &[i32]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::INT, "Uniform {name:?} expected `int` type in shader");
			unsafe { api::uniform1iv(u.location, data.len() as i32, data.as_ptr()) };
		}
	}
	fn ivec2(&mut self, name: &str, data: &[cvmath::Vec2<i32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::INT_VEC2, "Uniform {name:?} expected `ivec2` type in shader");
			unsafe { api::uniform2iv(u.location, data.len() as i32, data.as_ptr() as *const [i32; 2]) };
		}
	}
	fn ivec3(&mut self, name: &str, data: &[cvmath::Vec3<i32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::INT_VEC3, "Uniform {name:?} expected `ivec3` type in shader");
			unsafe { api::uniform3iv(u.location, data.len() as i32, data.as_ptr() as *const [i32; 3]) };
		}
	}
	fn ivec4(&mut self, name: &str, data: &[cvmath::Vec4<i32>]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::INT_VEC4, "Uniform {name:?} expected `ivec4` type in shader");
			unsafe { api::uniform4iv(u.location, data.len() as i32, data.as_ptr() as *const [i32; 4]) };
		}
	}
	fn mat2(&mut self, name: &str, data: &[cvmath::Mat2f]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_MAT2, "Uniform {name:?} expected `mat2` type in shader");
			for (i, data) in data.iter().enumerate() {
				let transposed = data.into_column_major();
				unsafe { api::uniformMatrix2fv(u.location + i as u32, 1, false, &transposed) };
			}
		}
	}
	fn mat3(&mut self, name: &str, data: &[cvmath::Mat3f]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_MAT3, "Uniform {name:?} expected `mat3` type in shader");
			for (i, data) in data.iter().enumerate() {
				let transposed = data.into_column_major();
				unsafe { api::uniformMatrix3fv(u.location + i as u32, 1, false, &transposed) };
			}
		}
	}
	fn mat4(&mut self, name: &str, data: &[cvmath::Mat4f]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_MAT4, "Uniform {name:?} expected `mat4` type in shader");
			for (i, data) in data.iter().enumerate() {
				let transposed = data.into_column_major();
				unsafe { api::uniformMatrix4fv(u.location + i as u32, 1, false, &transposed) };
			}
		}
	}
	fn transform2(&mut self, name: &str, data: &[cvmath::Transform2f]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_MAT3, "Uniform {name:?} expected `mat3` type in shader");
			for (i, data) in data.iter().enumerate() {
				let transposed = data.mat3().into_column_major();
				unsafe { api::uniformMatrix3fv(u.location + i as u32, 1, false, &transposed) };
			}
		}
	}
	fn transform3(&mut self, name: &str, data: &[cvmath::Transform3f]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::FLOAT_MAT4, "Uniform {name:?} expected `mat4` type in shader");
			for (i, data) in data.iter().enumerate() {
				let transposed = data.mat4().into_column_major();
				unsafe { api::uniformMatrix4fv(u.location + i as u32, 1, false, &transposed) };
			}
		}
	}
	fn sampler2d(&mut self, name: &str, textures: &[crate::Texture2D]) {
		if let Some(u) = self.shader.get_uniform(name) {
			debug_assert_eq!(u.ty, api::SAMPLER_2D, "Uniform {name:?} expected `sampler2D` type in shader");

			const MAX_TEXTURES: usize = 32;

			// Ensure we don't exceed the maximum number of texture units
			let base_unit = u.texture_unit as i32;
			assert!(base_unit as usize + textures.len() <= MAX_TEXTURES && textures.len() <= MAX_TEXTURES, "Too many textures! {name}");

			// Initialize texture unit assignments
			let mut units: [mem::MaybeUninit<i32>; MAX_TEXTURES] = [mem::MaybeUninit::uninit(); MAX_TEXTURES];
			let units = &mut units[..textures.len()];

			for (i, unit) in units.iter_mut().enumerate() {
				*unit = mem::MaybeUninit::new(base_unit + i as i32);
			}

			let units = unsafe { slice::from_raw_parts(units.as_ptr() as *const i32, units.len()) };

			// Upload all sampler indices in one call
			unsafe { api::uniform1iv(u.location, units.len() as i32, units.as_ptr()) };

			// Bind textures to texture units
			for (i, &id) in textures.iter().enumerate() {
				let texture = self.textures.get2d(id);
				let texture_unit = (base_unit + i as i32) as GLenum;
				unsafe {
					api::activeTexture(api::TEXTURE0 + texture_unit);
					api::bindTexture(api::TEXTURE_2D, texture.texture);
				}
			}
		}
	}
}

pub fn begin(this: &mut WebGLGraphics, args: &crate::RenderPassArgs) {
	if this.drawing {
		panic!("{}: draw call already in progress", name_of(&begin));
	}

	this.drawing = true;

	match args {
		&crate::RenderPassArgs::BackBuffer { ref viewport } => {
			gl_viewport(viewport);
		}
		&crate::RenderPassArgs::Immediate { color: _, depth: _, viewport: _ } => {
			unimplemented!("GlGraphics::begin with Immediate is not implemented yet");
		}
	}
}

pub fn clear(this: &mut WebGLGraphics, args: &crate::ClearArgs) {
	if !this.drawing {
		panic!("{}: called outside of an active draw call", name_of(&clear));
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
		unsafe { api::colorMask(api::TRUE, api::TRUE, api::TRUE, api::TRUE) };
		unsafe { api::clearColor(color.x, color.y, color.z, color.w) };
		mask |= api::COLOR_BUFFER_BIT;
	}
	if let Some(depth) = args.depth {
		unsafe { api::depthMask(api::TRUE) };
		unsafe { api::clearDepth(depth as f64) };
		mask |= api::DEPTH_BUFFER_BIT;
	}
	if let Some(stencil) = args.stencil{
		unsafe { api::stencilMask(0xff) };
		unsafe { api::clearStencil(stencil as i32) };
		mask |= api::STENCIL_BUFFER_BIT;
	}
	unsafe { api::clear(mask) };
}

pub fn arrays(this: &mut WebGLGraphics, args: &crate::DrawArgs) {
	this.metrics.draw_call_count += 1;
	this.metrics.vertex_count = u32::wrapping_add(this.metrics.vertex_count, args.vertex_end - args.vertex_start);

	if !this.drawing {
		panic!("{}: called outside of an active draw call", name_of(&clear));
	}

	for (i, data) in args.vertices.iter().enumerate() {
		assert!(this.vbuffers.get(data.buffer).is_some(), "{}: vertex buffer at index {} is invalid (handle: {:?})", name_of(&arrays), i, data.buffer);
	}
	let Some(shader) = this.shaders.get(args.shader) else { panic!("{}: invalid shader handle: {:?}", name_of(&arrays), args.shader); };

	assert!(args.vertex_end >= args.vertex_start, "{}: vertex_end ({}) < vertex_start ({})", name_of(&arrays), args.vertex_end, args.vertex_start);
	if args.vertex_start == args.vertex_end {
		return;
	}

	gl_draw_mask(&args.mask);
	gl_blend(args.blend_mode);
	gl_depth_test(args.depth_test);
	gl_cull_face(args.cull_mode);
	gl_scissor(&args.scissor);

	unsafe { api::useProgram(shader.program) };
	let enabled_attribs = gl_attributes(shader, args.vertices, &this.vbuffers);

	let ref mut set = WebGLUniformSetter { shader, textures: &this.textures };
	for uniforms in args.uniforms {
		uniforms.visit(set);
	}

	let mode = match args.prim_type {
		crate::PrimType::Lines => api::LINES,
		crate::PrimType::Triangles => api::TRIANGLES,
	};
	if args.instances >= 0 {
		unimplemented!("Instanced rendering is not supported in WebGL 1.0");
	}
	else {
		unsafe { api::drawArrays(mode, args.vertex_start as i32, (args.vertex_end - args.vertex_start) as i32) };
	}

	gl_attributes_disable(enabled_attribs);
	unsafe { api::useProgram(0) };
}

pub fn indexed(this: &mut WebGLGraphics, args: &crate::DrawIndexedArgs) {
	this.metrics.draw_call_count += 1;
	this.metrics.vertex_count = u32::wrapping_add(this.metrics.vertex_count, args.index_end - args.index_start);

	if !this.drawing {
		panic!("{}: called outside of an active draw call", name_of(&clear));
	}

	for (i, data) in args.vertices.iter().enumerate() {
		assert!(this.vbuffers.get(data.buffer).is_some(), "{}: vertex buffer at index {} is invalid (handle: {:?})", name_of(&arrays), i, data.buffer);
	}
	let Some(ib) = this.ibuffers.get(args.indices) else { panic!("{}: invalid index buffer handle: {:?}", name_of(&arrays), args.indices); };
	let Some(shader) = this.shaders.get(args.shader) else { panic!("{}: invalid shader handle: {:?}", name_of(&arrays), args.shader); };

	assert!(args.index_end >= args.index_start, "{}: index_end ({}) < index_start ({})", name_of(&indexed), args.index_end, args.index_start);
	if args.index_start == args.index_end {
		return;
	}

	gl_draw_mask(&args.mask);
	gl_blend(args.blend_mode);
	gl_depth_test(args.depth_test);
	gl_cull_face(args.cull_mode);
	gl_scissor(&args.scissor);

	unsafe { api::useProgram(shader.program) };
	unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, ib.buffer) };
	let enabled_attribs = gl_attributes(shader, args.vertices, &this.vbuffers);

	let ref mut set = WebGLUniformSetter { shader, textures: &this.textures };
	for uniforms in args.uniforms {
		uniforms.visit(set);
	}

	let mode = match args.prim_type {
		crate::PrimType::Lines => api::LINES,
		crate::PrimType::Triangles => api::TRIANGLES,
	};
	let count = args.index_end - args.index_start;
	let type_ = match ib.ty {
		crate::IndexType::U8 => api::UNSIGNED_BYTE,
		crate::IndexType::U16 => api::UNSIGNED_SHORT,
		crate::IndexType::U32 => api::UNSIGNED_INT,
	};
	let offset = args.index_start * ib.ty.size() as u32;
	if args.instances >= 0 {
		unimplemented!("Instanced rendering is not supported in WebGL 1.0");
	}
	else {
		unsafe { api::drawElements(mode, count as i32, type_, offset as api::types::GLintptr) };
	}

	gl_attributes_disable(enabled_attribs);
	unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, 0) };
	unsafe { api::useProgram(0) };
}

pub fn end(this: &mut WebGLGraphics) {
	this.drawing = false;
}
