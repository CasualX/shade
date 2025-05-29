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

fn gl_attributes(shader: &WebGLProgram, data: &[crate::DrawVertexBuffer], map: &ResourceMap<WebGLVertexBuffer>) {
	for vb in data {
		let Some(buf) = map.get(vb.buffer) else { continue }; // Validated in draw calls
		unsafe { api::bindBuffer(api::ARRAY_BUFFER, buf.buffer) };

		let layout = buf.layout;
		for attr in layout.attributes {
			let location = unsafe { api::getAttribLocation(shader.program, attr.name.as_ptr(), attr.name.len()) };
			if location < 0 {
				continue; // Attribute not found in shader
			}
			let location = location as u32;
			unsafe { api::enableVertexAttribArray(location) };
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
			if layout.size >= 256 {
				panic!("Vertex attribute size too large: {}", layout.size);
			}
			unsafe { api::vertexAttribPointer(location, size, type_, normalized, layout.size as GLsizei, attr.offset as GLintptr) };
		}
	}
}

fn gl_uniforms(uniforms: &[crate::UniformRef], shader: &WebGLProgram, textures: &WebGLTextures) {
	for uniform_ref in uniforms {
		for uattr in uniform_ref.layout.fields {
			let data_ptr = unsafe { uniform_ref.data_ptr.offset(uattr.offset as isize) };
			if let Some(u) = shader.get_uniform(uattr.name) {
				// println!("Uniform: {} (index: {})", uattr.name, i);
				let location = u.location;
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
					crate::UniformType::Mat2x2 { order } => {
						let transposed;
						let data_ptr = match order {
							crate::MatrixLayout::ColumnMajor => data_ptr,
							crate::MatrixLayout::RowMajor => {
								let data = &unsafe { *(data_ptr as *const [f32; 4]) };
								transposed = [
									data[0], data[2], // first column
									data[1], data[3], // second column
								];
								&transposed as *const _ as *const u8
							}
						};
						unsafe { api::uniformMatrix2fv(location, uattr.len as i32, false, data_ptr as *const _) }
					},
					crate::UniformType::Mat3x3 { order } => {
						let transposed;
						let data_ptr = match order {
							crate::MatrixLayout::ColumnMajor => data_ptr,
							crate::MatrixLayout::RowMajor => {
								let data = &unsafe { *(data_ptr as *const [f32; 9]) };
								transposed = [
									data[0], data[3], data[6],
									data[1], data[4], data[7],
									data[2], data[5], data[8],
								];
								&transposed as *const _ as *const u8
							}
						};
						unsafe { api::uniformMatrix3fv(location, uattr.len as i32, false, data_ptr as *const _) }
					},
					crate::UniformType::Mat4x4 { order } => {
						let transposed;
						let data_ptr = match order {
							crate::MatrixLayout::ColumnMajor => data_ptr,
							crate::MatrixLayout::RowMajor => {
								let data = &unsafe { *(data_ptr as *const [f32; 16]) };
								transposed = [
									data[0],  data[4],  data[8],  data[12],
									data[1],  data[5],  data[9],  data[13],
									data[2],  data[6],  data[10], data[14],
									data[3],  data[7],  data[11], data[15],
								];
								&transposed as *const _ as *const u8
							}
						};
						unsafe { api::uniformMatrix4fv(location, uattr.len as i32, false, data_ptr as *const _) }
					},
					crate::UniformType::Sampler2D => {
						let ids = unsafe { slice::from_raw_parts(data_ptr as *const crate::Texture2D, uattr.len as usize) };
						for &id in ids {
							let texture = textures.get2d(id);
							let texture_unit = u.texture_unit as i32 & 0x1f;
							unsafe { api::uniform1iv(location, 1, &texture_unit) };
							unsafe { api::activeTexture(api::TEXTURE0 + texture_unit as u32) };
							unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
						}
					}
					_ => unimplemented!("Uniform type not implemented: {:?}", uattr.ty),
				}
			}
			else {
				// panic!("Uniform not found: {}", uattr.name);
			}
		}
	}
}

pub fn begin(this: &mut WebGLGraphics) -> Result<(), crate::GfxError> {
	if this.drawing {
		return Err(crate::GfxError::InvalidDrawCallTime);
	}

	this.drawing = true;
	Ok(())
}

pub fn clear(this: &mut WebGLGraphics, args: &crate::ClearArgs) -> Result<(), crate::GfxError> {
	if !this.drawing {
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

	Ok(())
}

pub fn arrays(this: &mut WebGLGraphics, args: &crate::DrawArgs) -> Result<(), crate::GfxError> {
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

	unsafe { api::useProgram(shader.program) };
	gl_attributes(shader, args.vertices, &this.vbuffers);
	gl_uniforms(args.uniforms, shader, &this.textures);

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

	Ok(())
}

pub fn indexed(this: &mut WebGLGraphics, args: &crate::DrawIndexedArgs) -> Result<(), crate::GfxError> {
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

	unsafe { api::useProgram(shader.program) };
	unsafe { api::bindBuffer(api::ELEMENT_ARRAY_BUFFER, ib.buffer) };
	gl_attributes(shader, args.vertices, &this.vbuffers);
	gl_uniforms(args.uniforms, shader, &this.textures);

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

	Ok(())
}

pub fn end(this: &mut WebGLGraphics) -> Result<(), crate::GfxError> {
	this.drawing = false;
	Ok(())
}
