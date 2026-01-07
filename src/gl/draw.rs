use super::*;

fn gl_draw_mask(mask: &crate::DrawMask) {
	let red = if mask.red { gl::TRUE } else { gl::FALSE };
	let green = if mask.green { gl::TRUE } else { gl::FALSE };
	let blue = if mask.blue { gl::TRUE } else { gl::FALSE };
	let alpha = if mask.alpha { gl::TRUE } else { gl::FALSE };
	gl_check!(gl::ColorMask(red, green, blue, alpha));
	let depth = if mask.depth { gl::TRUE } else { gl::FALSE };
	gl_check!(gl::DepthMask(depth));
}

fn gl_blend(blend_mode: crate::BlendMode) {
	struct GlBlend {
		sfactor: GLenum,
		dfactor: GLenum,
		equation: GLenum,
	}
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
		crate::BlendMode::PremultipliedAlpha => GlBlend {
			sfactor: gl::ONE,
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

fn gl_scissor(scissor: &Option<cvmath::Bounds2<i32>>) {
	if let Some(scissor) = scissor {
		gl_check!(gl::Enable(gl::SCISSOR_TEST));
		gl_check!(gl::Scissor(scissor.mins.x, scissor.mins.y, scissor.width(), scissor.height()));
	}
	else {
		gl_check!(gl::Disable(gl::SCISSOR_TEST));
	}
}

fn gl_viewport(viewport: &cvmath::Bounds2<i32>) {
	gl_check!(gl::Viewport(viewport.mins.x, viewport.mins.y, viewport.width(), viewport.height()));
}

fn gl_attributes(shader: &GlShader, data: &[crate::DrawVertexBuffer], map: &ResourceMap<GlVertexBuffer>) -> u32 {
	let mut enabled_attribs = 0u32;
	for vb in data {
		let Some(buf) = map.get(vb.buffer) else { continue }; // Validated in draw calls
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));

		let layout = buf.layout;
		for attr in layout.attributes {
			let Some(attrib) = shader.attribs.get(attr.name) else { continue };
			enabled_attribs |= 1 << attrib.location;
			gl_check!(gl::EnableVertexAttribArray(attrib.location));

			let size = attr.format.size() as GLint;
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
			gl_check!(gl::VertexAttribPointer(attrib.location, size, type_, normalized, layout.size as i32, attr.offset as usize as *const GLvoid));

			let divisor = match vb.divisor {
				crate::VertexDivisor::PerVertex => 0,
				crate::VertexDivisor::PerInstance => 1,
			};
			gl_check!(gl::VertexAttribDivisor(attrib.location, divisor));
		}
	}
	gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));

	// Assert that all attributes are bound
	#[cfg(debug_assertions)]
	for (name, attr) in &shader.attribs {
		assert!(enabled_attribs & (1 << attr.location) != 0, "Attribute {} not bound in shader {}", name, shader.program);
	}
	return enabled_attribs;
}

fn gl_attributes_disable(enabled_attribs: u32) {
	for i in 0..32 {
		if enabled_attribs & (1 << i) != 0 {
			gl_check!(gl::DisableVertexAttribArray(i as u32));
		}
	}
}

struct GlUniformSetter<'a> {
	shader: &'a GlShader,
	textures: &'a GlTextures,
}
impl<'a> crate::UniformSetter for GlUniformSetter<'a> {
	fn float(&mut self, name: &str, data: &[f32]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT, "Uniform {name:?} expected `float` type in shader");
			gl_check!(gl::Uniform1fv(u.location, data.len() as i32, data.as_ptr()));
		}
	}
	fn vec2(&mut self, name: &str, data: &[cvmath::Vec2<f32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_VEC2, "Uniform {name:?} expected `vec2` type in shader");
			gl_check!(gl::Uniform2fv(u.location, data.len() as i32, data.as_ptr() as *const f32));
		}
	}
	fn vec3(&mut self, name: &str, data: &[cvmath::Vec3<f32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_VEC3, "Uniform {name:?} expected `vec3` type in shader");
			gl_check!(gl::Uniform3fv(u.location, data.len() as i32, data.as_ptr() as *const f32));
		}
	}
	fn vec4(&mut self, name: &str, data: &[cvmath::Vec4<f32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_VEC4, "Uniform {name:?} expected `vec4` type in shader");
			gl_check!(gl::Uniform4fv(u.location, data.len() as i32, data.as_ptr() as *const f32));
		}
	}
	fn int(&mut self, name: &str, data: &[i32]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::INT, "Uniform {name:?} expected `int` type in shader");
			gl_check!(gl::Uniform1iv(u.location, data.len() as i32, data.as_ptr()));
		}
	}
	fn ivec2(&mut self, name: &str, data: &[cvmath::Vec2<i32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::INT_VEC2, "Uniform {name:?} expected `ivec2` type in shader");
			gl_check!(gl::Uniform2iv(u.location, data.len() as i32, data.as_ptr() as *const i32));
		}
	}
	fn ivec3(&mut self, name: &str, data: &[cvmath::Vec3<i32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::INT_VEC3, "Uniform {name:?} expected `ivec3` type in shader");
			gl_check!(gl::Uniform3iv(u.location, data.len() as i32, data.as_ptr() as *const i32));
		}
	}
	fn ivec4(&mut self, name: &str, data: &[cvmath::Vec4<i32>]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::INT_VEC4, "Uniform {name:?} expected `ivec4` type in shader");
			gl_check!(gl::Uniform4iv(u.location, data.len() as i32, data.as_ptr() as *const i32));
		}
	}
	fn mat2(&mut self, name: &str, data: &[cvmath::Mat2f]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_MAT2, "Uniform {name:?} expected `mat2` type in shader");
			gl_check!(gl::UniformMatrix2fv(u.location, data.len() as i32, gl::TRUE, data.as_ptr() as *const f32));
		}
	}
	fn mat3(&mut self, name: &str, data: &[cvmath::Mat3f]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_MAT3, "Uniform {name:?} expected `mat3` type in shader");
			gl_check!(gl::UniformMatrix3fv(u.location, data.len() as i32, gl::TRUE, data.as_ptr() as *const f32));
		}
	}
	fn mat4(&mut self, name: &str, data: &[cvmath::Mat4f]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_MAT4, "Uniform {name:?} expected `mat4` type in shader");
			gl_check!(gl::UniformMatrix4fv(u.location, data.len() as i32, gl::TRUE, data.as_ptr() as *const f32));
		}
	}
	fn transform2(&mut self, name: &str, data: &[cvmath::Transform2f]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_MAT3x2, "Uniform {name:?} expected `mat3x2` type in shader");
			gl_check!(gl::UniformMatrix3x2fv(u.location, data.len() as i32, gl::TRUE, data.as_ptr() as *const f32));
		}
	}
	fn transform3(&mut self, name: &str, data: &[cvmath::Transform3f]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::FLOAT_MAT4x3, "Uniform {name:?} expected `mat4x3` type in shader");
			gl_check!(gl::UniformMatrix4x3fv(u.location, data.len() as i32, gl::TRUE, data.as_ptr() as *const f32));
		}
	}
	fn sampler2d(&mut self, name: &str, textures: &[crate::Texture2D]) {
		if let Some(u) = self.shader.uniforms.get(name) {
			debug_assert_eq!(u.ty, gl::SAMPLER_2D, "Uniform {name:?} expected `sampler2D` type in shader");

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

			// Upload all sampler indices at once to the uniform array
			gl_check!(gl::Uniform1iv(u.location, units.len() as i32, units.as_ptr()));

			// Bind textures to texture units
			for (i, &id) in textures.iter().enumerate() {
				let texture = self.textures.get2d(id);
				assert!(texture.info.props.usage.has(crate::TextureUsage::SAMPLED), "Texture was not created with SAMPLED usage");
				let texture_unit = (base_unit + i as i32) as u32;
				gl_check!(gl::ActiveTexture(gl::TEXTURE0 + texture_unit));
				gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
			}
		}
	}
}

fn gl_immediate_fbo(this: &mut GlGraphics, color: &[crate::Texture2D], depth: crate::Texture2D) {
	// Create a temporary framebuffer
	let mut fbo = 0;
	gl_check!(gl::GenFramebuffers(1, &mut fbo));
	gl_check!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));

	// Attach color textures
	assert!(color.len() <= 16, "Immediate mode framebuffer cannot have more than 16 color attachments");
	let mut draw_buffers: [GLenum; 16] = [gl::NONE; 16];
	for (i, &tex_id) in color.iter().enumerate() {
		let texture = this.textures.get2d(tex_id);
		assert!(texture.info.props.usage.has(crate::TextureUsage::COLOR_TARGET), "Texture was not created with COLOR_TARGET usage");
		gl_check!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, gl::TEXTURE_2D, texture.texture, 0));
		draw_buffers[i] = gl::COLOR_ATTACHMENT0 + i as u32;
	}

	// Specify which color attachments to draw to
	if color.is_empty() {
		gl_check!(gl::DrawBuffer(gl::NONE));
		gl_check!(gl::ReadBuffer(gl::NONE));
	}
	else {
		gl_check!(gl::DrawBuffers(color.len() as i32, draw_buffers.as_ptr()));
	}

	// Attach depth texture if valid
	if depth != crate::Texture2D::INVALID {
		let depth_tex = this.textures.get2d(depth);
		assert!(depth_tex.info.props.usage.has(crate::TextureUsage::DEPTH_STENCIL_TARGET), "Texture was not created with DEPTH_STENCIL_TARGET usage");
		gl_check!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_tex.texture, 0));
	}

	// Check framebuffer completeness
	#[cfg(debug_assertions)] {
		let status = gl_check!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
		if status != gl::FRAMEBUFFER_COMPLETE {
			panic!("Immediate framebuffer is incomplete: status = 0x{:X}", status);
		}
	}

	this.immediate_fbo = Some(fbo);
}

pub fn begin(this: &mut GlGraphics, args: &crate::RenderPassArgs) {
	this.draw_begin = time::Instant::now();

	if this.drawing {
		panic!("{}: draw call already in progress", name_of(&begin));
	}

	this.drawing = true;

	match args {
		&crate::RenderPassArgs::BackBuffer { ref viewport } => {
			gl_check!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
			gl_viewport(viewport);
		}
		&crate::RenderPassArgs::Immediate { color, depth, ref viewport } => {
			gl_immediate_fbo(this, color, depth);
			gl_viewport(viewport);
		}
	}
}

pub fn clear(this: &mut GlGraphics, args: &crate::ClearArgs) {
	if !this.drawing {
		panic!("{}: called outside of an active draw call", name_of(&clear));
	}

	gl_scissor(&args.scissor);

	let mut mask = 0;
	if let Some(color) = args.color {
		gl_check!(gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE));
		gl_check!(gl::ClearColor(color.x, color.y, color.z, color.w));
		mask |= gl::COLOR_BUFFER_BIT;
	}
	if let Some(depth) = args.depth {
		gl_check!(gl::DepthMask(gl::TRUE));
		gl_check!(gl::ClearDepth(depth as f64));
		mask |= gl::DEPTH_BUFFER_BIT;
	}
	if let Some(stencil) = args.stencil{
		gl_check!(gl::StencilMask(0xff));
		gl_check!(gl::ClearStencil(stencil as GLint));
		mask |= gl::STENCIL_BUFFER_BIT;
	}
	gl_check!(gl::Clear(mask));
}

pub fn arrays(this: &mut GlGraphics, args: &crate::DrawArgs) {
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

	gl_check!(gl::UseProgram(shader.program));
	gl_check!(gl::BindVertexArray(this.dynamic_vao));
	let enabled_attribs = gl_attributes(shader, args.vertices, &this.vbuffers);

	let ref mut set = GlUniformSetter { shader, textures: &this.textures };
	for uniforms in args.uniforms {
		uniforms.visit(set);
	}

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

	gl_attributes_disable(enabled_attribs);
	gl_check!(gl::BindVertexArray(0));
	gl_check!(gl::UseProgram(0));
}

pub fn indexed(this: &mut GlGraphics, args: &crate::DrawIndexedArgs) {
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

	gl_check!(gl::UseProgram(shader.program));
	gl_check!(gl::BindVertexArray(this.dynamic_vao));
	gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.buffer));
	gl_attributes(shader, args.vertices, &this.vbuffers);

	let ref mut set = GlUniformSetter { shader, textures: &this.textures };
	for uniforms in args.uniforms {
		uniforms.visit(set);
	}

	let mode = match args.prim_type {
		crate::PrimType::Lines => gl::LINES,
		crate::PrimType::Triangles => gl::TRIANGLES,
	};
	let count = args.index_end - args.index_start;
	let type_ = match ib.ty {
		crate::IndexType::U8 => gl::UNSIGNED_BYTE,
		crate::IndexType::U16 => gl::UNSIGNED_SHORT,
		crate::IndexType::U32 => gl::UNSIGNED_INT,
	};
	let offset = args.index_start * ib.ty.size() as u32;
	if args.instances >= 0 {
		gl_check!(gl::DrawElementsInstanced(mode, count as i32, type_, offset as *const GLvoid, args.instances));
	}
	else {
		gl_check!(gl::DrawElements(mode, count as i32, type_, offset as *const GLvoid));
	}

	gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
	gl_check!(gl::BindVertexArray(0));
	gl_check!(gl::UseProgram(0));
}

pub fn end(this: &mut GlGraphics) {
	this.metrics.draw_duration += this.draw_begin.elapsed();
	this.drawing = false;

	// Clean up immediate framebuffer if one was created
	if let Some(fbo) = this.immediate_fbo.take() {
		gl_check!(gl::DeleteFramebuffers(1, &fbo));
	}

	// Unbind framebuffer to return to default state
	gl_check!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
}
