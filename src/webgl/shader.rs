use super::*;

pub fn create(this: &mut WebGLGraphics, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Result<crate::Shader, crate::GfxError> {
	let program = unsafe { api::createProgram() };
	let id = this.shaders.insert(name, WebGLProgram { program, uniforms: Vec::new() });


	let Some(shader) = this.shaders.get_mut(id) else { return Err(crate::GfxError::InvalidHandle) };
	let mut success = true;

	shader.uniforms.clear();

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

	if success {
		unsafe { api::attachShader(shader.program, vertex_shader) };
		unsafe { api::attachShader(shader.program, fragment_shader) };
		unsafe { api::linkProgram(shader.program) };
		let status = unsafe { api::getProgramParameter(shader.program, api::LINK_STATUS) };
		if status == 0 {
			unsafe { api::getProgramInfoLog(shader.program) };
			success = false;
		}
		else {
			unsafe { api::useProgram(shader.program) };
			let count = unsafe { api::getProgramParameter(shader.program, api::ACTIVE_UNIFORMS) };
			let mut texture_slot = -1;
			for i in 0..count {
				let mut name_len = 0;
				let mut size = 0;
				let mut ty = 0;
				let mut name = [0; 64];
				unsafe { api::getActiveUniform(shader.program, i as u32, 64, &mut name_len, &mut size, &mut ty, name.as_mut_ptr()) };
				let location = unsafe { api::getUniformLocation(shader.program, name.as_ptr(), name_len as usize) };
				let texture_unit = if ty == api::SAMPLER_2D || ty == api::SAMPLER_CUBE {
					texture_slot += 1;
					texture_slot
				} else { -1 };
				shader.uniforms.push(WebGLActiveUniform {
					location,
					namelen: name_len as u8,
					namebuf: name,
					_size: size,
					_ty: ty,
					texture_unit,
				});
			}
		}
	}

	unsafe { api::deleteShader(vertex_shader) };
	unsafe { api::deleteShader(fragment_shader) };
	return if success { Ok(id) } else { Err(crate::GfxError::ShaderCompileError) };
}

pub fn find(this: &mut WebGLGraphics, name: &str) -> Result<crate::Shader, crate::GfxError> {
	this.shaders.find_id(name).ok_or(crate::GfxError::NameNotFound)
}

pub fn delete(this: &mut WebGLGraphics, id: crate::Shader) -> Result<(), crate::GfxError> {
	todo!()
}
