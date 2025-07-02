use super::*;

pub fn create(this: &mut WebGLGraphics, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Result<crate::Shader, crate::GfxError> {
	let mut success = true;

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

	if !success {
		unsafe { api::deleteShader(vertex_shader) };
		unsafe { api::deleteShader(fragment_shader) };
		return Err(crate::GfxError::ShaderCompileError);
	}

	let program = unsafe { api::createProgram() };

	unsafe { api::attachShader(program, vertex_shader) };
	unsafe { api::attachShader(program, fragment_shader) };
	unsafe { api::linkProgram(program) };
	unsafe { api::deleteShader(vertex_shader) };
	unsafe { api::deleteShader(fragment_shader) };

	let status = unsafe { api::getProgramParameter(program, api::LINK_STATUS) };
	if status == 0 {
		unsafe { api::getProgramInfoLog(program) };
		unsafe { api::deleteProgram(program) };
		return Err(crate::GfxError::ShaderCompileError);
	}

	let nattribs = unsafe { api::getProgramParameter(program, api::ACTIVE_ATTRIBUTES) };
	let mut attribs = Vec::new();
	for i in 0..nattribs {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 63], 0, 0, 0);
		unsafe { api::getActiveAttrib(program, i as u32, namebuf.len() as i32 - 1, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr()) };

		let location = unsafe { api::getAttribLocation(program, namebuf.as_ptr(), namelen as usize) };
		debug_assert!(location >= 0, "Invalid attribute location: {}", String::from_utf8_lossy(&namebuf[..namelen as usize]));
		let location = location as GLuint;
		let namelen = namelen as u8;
		attribs.push(WebGLActiveAttrib { location, size, ty, namelen, namebuf });
	}

	let nuniforms = unsafe { api::getProgramParameter(program, api::ACTIVE_UNIFORMS) };
	let mut uniforms = Vec::new();
	let mut texture_slot = -1;
	for i in 0..nuniforms {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 66], 0, 0, 0);
		unsafe { api::getActiveUniform(program, i as u32, namebuf.len() as i32 - 1, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr()) };

		let location = unsafe { api::getUniformLocation(program, namebuf.as_ptr(), namelen as usize) };
		assert!(location >= 0, "Invalid uniform location: {}", String::from_utf8_lossy(&namebuf[..namelen as usize]));

		let location = location as GLuint;
		let needs_texture_unit = matches!(ty, api::SAMPLER_2D | api::SAMPLER_CUBE);
		let texture_unit = if needs_texture_unit { texture_slot += 1; texture_slot } else { -1 };
		let namelen = namelen as u8;
		uniforms.push(WebGLActiveUniform { location, size, ty, texture_unit, namelen, namebuf });
	}

	let id = this.shaders.insert(name, WebGLProgram { program, attribs, uniforms });
	Ok(id)
}

pub fn find(this: &mut WebGLGraphics, name: &str) -> Result<crate::Shader, crate::GfxError> {
	this.shaders.find_id(name).ok_or(crate::GfxError::NameNotFound)
}

pub fn delete(this: &mut WebGLGraphics, id: crate::Shader) {
	let Some(shader) = this.shaders.remove(id) else { return };
	unsafe { api::deleteProgram(shader.program) };
}
