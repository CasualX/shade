use super::*;

pub fn create(this: &mut WebGLGraphics, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> crate::Shader {
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
		return crate::Shader::INVALID;
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
		return crate::Shader::INVALID;
	}

	let nattribs = unsafe { api::getProgramParameter(program, api::ACTIVE_ATTRIBUTES) };
	let mut attribs = HashMap::new();
	for i in 0..nattribs {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 64], 0, 0, 0);
		unsafe { api::getActiveAttrib(program, i as u32, namebuf.len() as i32 - 1, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr()) };
		let name = str::from_utf8(&namebuf[..namelen as usize]).unwrap();

		let location = unsafe { api::getAttribLocation(program, name.as_ptr(), name.len()) };
		debug_assert!(location >= 0, "Invalid attribute location: {}", name);
		let location = location as GLuint;
		attribs.insert(name.into(), WebGLActiveAttrib { location, size, ty });
	}

	let nuniforms = unsafe { api::getProgramParameter(program, api::ACTIVE_UNIFORMS) };
	let mut uniforms = HashMap::new();
	let mut texture_slot = -1;
	for i in 0..nuniforms {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 64], 0, 0, 0);
		unsafe { api::getActiveUniform(program, i as u32, namebuf.len() as i32 - 1, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr()) };
		let name = str::from_utf8(&namebuf[..namelen as usize]).unwrap();

		let location = unsafe { api::getUniformLocation(program, name.as_ptr(), name.len()) };
		assert!(location >= 0, "Invalid uniform location: {}", name);

		let location = location as GLuint;
		let needs_texture_unit = matches!(ty, api::SAMPLER_2D | api::SAMPLER_CUBE);
		let texture_unit = if needs_texture_unit { texture_slot += 1; texture_slot } else { -1 };
		uniforms.insert(name.into(), WebGLActiveUniform { location, size, ty, texture_unit });
	}

	return this.shaders.insert(name, WebGLProgram { program, attribs, uniforms });
}

pub fn find(this: &mut WebGLGraphics, name: &str) -> crate::Shader {
	this.shaders.find_id(name).unwrap_or(crate::Shader::INVALID)
}

pub fn delete(this: &mut WebGLGraphics, id: crate::Shader) {
	let Some(shader) = this.shaders.remove(id) else { return };
	unsafe { api::deleteProgram(shader.program) };
}
