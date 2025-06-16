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

fn gl_attributes(shader: &GlShader, data: &[crate::DrawVertexBuffer], map: &ResourceMap<GlVertexBuffer>) {
	let mut attribs = 0u64;
	for dvb in data {
		let Some(buf) = map.get(dvb.buffer) else { continue }; // Validated in draw calls
		gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, buf.buffer));

		let layout = buf.layout;
		for attr in layout.attributes {
			let mut namebuf = [0u8; 64];
			namebuf[..attr.name.len()].copy_from_slice(attr.name.as_bytes());
			namebuf[attr.name.len()] = 0; // Null-terminate the string
			let location: i32 = gl_check!(gl::GetAttribLocation(shader.program, namebuf.as_ptr() as *const _));
			// This program does not use this attribute, skip it
			if location < 0 {
				continue;
			}
			let location = location as u32;
			attribs |= 1 << location;
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
			gl_check!(gl::VertexAttribPointer(location, size, type_, normalized, layout.size as i32, attr.offset as usize as *const GLvoid));
			let divisor = match dvb.divisor {
				crate::VertexDivisor::PerVertex => 0,
				crate::VertexDivisor::PerInstance => 1,
			};
			gl_check!(gl::VertexAttribDivisor(location, divisor));
			gl_check!(gl::EnableVertexAttribArray(location));
		}
	}

	gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));

	// Assert that all attributes are bound
	for attr in &shader.attribs {
		assert!(attribs & (1 << attr.location) != 0, "Attribute {} not bound in shader {}", attr.name(), shader.program);
	}
}

fn gl_transpose(layout: crate::MatrixLayout) -> GLboolean {
	match layout {
		crate::MatrixLayout::ColumnMajor => gl::FALSE,
		crate::MatrixLayout::RowMajor => gl::TRUE,
	}
}

fn gl_uniforms(uniforms: &[crate::UniformRef], shader: &GlShader, textures: &GlTextures) {
	gl_check!(gl::UseProgram(shader.program));

	for uniform_ref in uniforms {
		for uattr in uniform_ref.layout.fields {
			let data_ptr = unsafe { uniform_ref.data_ptr.offset(uattr.offset as isize) };
			if let Some(u) = shader.get_uniform(uattr.name) {
				// println!("Uniform: {} (index: {})", uattr.name, i);
				let location = u.location;
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
					crate::UniformType::Mat2x2 { layout } => gl_check!(gl::UniformMatrix2fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat2x3 { layout } => gl_check!(gl::UniformMatrix2x3fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat2x4 { layout } => gl_check!(gl::UniformMatrix2x4fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat3x2 { layout } => gl_check!(gl::UniformMatrix3x2fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat3x3 { layout } => gl_check!(gl::UniformMatrix3fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat3x4 { layout } => gl_check!(gl::UniformMatrix3x4fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat4x2 { layout } => gl_check!(gl::UniformMatrix4x2fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat4x3 { layout } => gl_check!(gl::UniformMatrix4x3fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Mat4x4 { layout } => gl_check!(gl::UniformMatrix4fv(location, uattr.len as i32, gl_transpose(layout), data_ptr as *const _)),
					crate::UniformType::Sampler2D => {
						let ids = unsafe { slice::from_raw_parts(data_ptr as *const crate::Texture2D, uattr.len as usize) };
						for &id in ids {
							let texture = textures.get2d(id);
							let texture_unit = u.texture_unit & 0x1F; // Ensure it's in the range 0-31
							gl_check!(gl::Uniform1i(location, texture_unit as i32));
							gl_check!(gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32));
							gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
						}
					}
				}
			}
			else {
				// panic!("Uniform not found: {}", uattr.name);
			}
		}
	}
}

pub fn begin(this: &mut GlGraphics) -> Result<(), crate::GfxError> {
	if this.drawing {
		return Err(crate::GfxError::InvalidDrawCallTime);
	}

	this.drawing = true;
	Ok(())
}

pub fn clear(this: &mut GlGraphics, args: &crate::ClearArgs) -> Result<(), crate::GfxError> {
	if !this.drawing {
		return Err(crate::GfxError::InvalidDrawCallTime);
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

	Ok(())
}

pub fn arrays(this: &mut GlGraphics, args: &crate::DrawArgs) -> Result<(), crate::GfxError> {
	if !this.drawing {
		return Err(crate::GfxError::InvalidDrawCallTime);
	}

	for data in args.vertices {
		let Some(_) = this.vbuffers.get(data.buffer) else { return Err(crate::GfxError::InvalidHandle) };
	}
	let Some(shader) = this.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidHandle) };

	if args.vertex_end < args.vertex_start {
		return Err(crate::GfxError::IndexOutOfBounds);
	}
	if args.vertex_start == args.vertex_end {
		return Ok(());
	}

	gl_draw_mask(&args.mask);
	gl_blend(args.blend_mode);
	gl_depth_test(args.depth_test);
	gl_cull_face(args.cull_mode);
	gl_scissor(&args.scissor);
	gl_viewport(&args.viewport);

	gl_check!(gl::UseProgram(shader.program));
	gl_check!(gl::BindVertexArray(this.dynamic_vao));
	gl_attributes(shader, args.vertices, &this.vbuffers);
	gl_uniforms(args.uniforms, shader, &this.textures);

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

	gl_check!(gl::BindVertexArray(0));
	gl_check!(gl::UseProgram(0));

	Ok(())
}

pub fn indexed(this: &mut GlGraphics, args: &crate::DrawIndexedArgs) -> Result<(), crate::GfxError> {
	if !this.drawing {
		return Err(crate::GfxError::InvalidDrawCallTime);
	}

	for data in args.vertices {
		let Some(_) = this.vbuffers.get(data.buffer) else { return Err(crate::GfxError::InvalidHandle) };
	}
	let Some(ib) = this.ibuffers.get(args.indices) else { return Err(crate::GfxError::InvalidHandle) };
	let Some(shader) = this.shaders.get(args.shader) else { return Err(crate::GfxError::InvalidHandle) };

	if args.index_end < args.index_start || args.vertex_end < args.vertex_start {
		return Err(crate::GfxError::IndexOutOfBounds);
	}
	if args.index_start == args.index_end {
		return Ok(());
	}

	gl_draw_mask(&args.mask);
	gl_blend(args.blend_mode);
	gl_depth_test(args.depth_test);
	gl_cull_face(args.cull_mode);
	gl_scissor(&args.scissor);
	gl_viewport(&args.viewport);

	gl_check!(gl::UseProgram(shader.program));
	gl_check!(gl::BindVertexArray(this.dynamic_vao));
	gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.buffer));
	gl_attributes(shader, args.vertices, &this.vbuffers);
	gl_uniforms(args.uniforms, shader, &this.textures);

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

	Ok(())
}

pub fn end(this: &mut GlGraphics) -> Result<(), crate::GfxError> {
	this.drawing = false;
	Ok(())
}
