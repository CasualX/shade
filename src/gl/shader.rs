use super::*;

pub fn find(this: &mut GlGraphics, name: &str) -> Result<crate::Shader, crate::GfxError> {
	this.shaders.find_id(name).ok_or(crate::GfxError::NameNotFound)
}

pub fn create(this: &mut GlGraphics, name: Option<&str>, vertex_source: &str, fragment_source: &str) -> Result<crate::Shader, crate::GfxError> {
	let mut success = true;
	let mut status = 0;

	let vertex_shader = gl_check!(gl::CreateShader(gl::VERTEX_SHADER));
	gl_check!(gl::ShaderSource(vertex_shader, 1, &(vertex_source.as_ptr() as *const _), &(vertex_source.len() as GLint)));
	gl_check!(gl::CompileShader(vertex_shader));
	gl_check!(gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut status));
	if status == 0 {
		let mut log_len = 0;
		gl_check!(gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_len));
		let mut log = vec![0; log_len as usize];
		gl_check!(gl::GetShaderInfoLog(vertex_shader, log_len, ptr::null_mut::<GLsizei>(), log.as_mut_ptr() as *mut GLchar));
		println!("# Vertex shader compile log:\n{}", String::from_utf8_lossy(&log));
		success = false;
	}

	let fragment_shader = gl_check!(gl::CreateShader(gl::FRAGMENT_SHADER));
	gl_check!(gl::ShaderSource(fragment_shader, 1, &(fragment_source.as_ptr() as *const _), &(fragment_source.len() as GLint)));
	gl_check!(gl::CompileShader(fragment_shader));
	gl_check!(gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut status));
	if status == 0 {
		let mut log_len = 0;
		gl_check!(gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_len));
		let mut log = vec![0; log_len as usize];
		gl_check!(gl::GetShaderInfoLog(fragment_shader, log_len, ptr::null_mut::<GLsizei>(), log.as_mut_ptr() as *mut GLchar));
		println!("# Fragment shader compile log:\n{}", String::from_utf8_lossy(&log));
		success = false;
	}

	if !success {
		gl_check!(gl::DeleteShader(vertex_shader));
		gl_check!(gl::DeleteShader(fragment_shader));
		return Err(crate::GfxError::ShaderCompileError);
	}

	let program = gl_check!(gl::CreateProgram());

	gl_check!(gl::AttachShader(program, vertex_shader));
	gl_check!(gl::AttachShader(program, fragment_shader));
	gl_check!(gl::LinkProgram(program));
	gl_check!(gl::GetProgramiv(program, gl::LINK_STATUS, &mut status));
	gl_check!(gl::DeleteShader(vertex_shader));
	gl_check!(gl::DeleteShader(fragment_shader));

	if status == 0 {
		let mut log_len = 0;
		gl_check!(gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len));
		let mut log = vec![0; log_len as usize];
		gl_check!(gl::GetProgramInfoLog(program, log_len, ptr::null_mut::<GLsizei>(), log.as_mut_ptr() as *mut GLchar));
		println!("# Program link log:\n{}", String::from_utf8_lossy(&log));
		gl_check!(gl::DeleteProgram(program));
		return Err(crate::GfxError::ShaderCompileError);
	}

	let mut nattribs = 0;
	gl_check!(gl::GetProgramiv(program, gl::ACTIVE_ATTRIBUTES, &mut nattribs));
	let mut attribs = Vec::new();
	for i in 0..nattribs {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 64], 0, 0, 0);
		gl_check!(gl::GetActiveAttrib(program, i as u32, namebuf.len() as GLsizei, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr() as *mut GLchar));
		assert!((namelen as usize) < namebuf.len(), "Attribute name too long: {}", String::from_utf8_lossy(&namebuf));

		let location = gl_check!(gl::GetAttribLocation(program, namebuf.as_ptr() as *const _));
		assert!(location >= 0, "Attribute not found?!: {}", String::from_utf8_lossy(&namebuf));

		let location = location as u32;
		let namelen = namelen as u8;
		attribs.push(GlActiveAttrib { location, size, ty, namelen, namebuf });
		// println!("Attribute: {} (location: {})", shader.attribs.last().unwrap().name(), location);
	}

	let mut nuniforms = 0;
	gl_check!(gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut nuniforms));
	let mut uniforms = Vec::new();
	let mut texture_slot = -1;
	for i in 0..nuniforms {
		let (mut namebuf, mut namelen, mut size, mut ty) = ([0; 64], 0, 0, 0);
		gl_check!(gl::GetActiveUniform(program, i as u32, namebuf.len() as GLsizei, &mut namelen, &mut size, &mut ty, namebuf.as_mut_ptr() as *mut GLchar));
		assert!((namelen as usize) < namebuf.len(), "Uniform name too long: {}", String::from_utf8_lossy(&namebuf));

		let location = gl_check!(gl::GetUniformLocation(program, namebuf.as_ptr() as *const _));
		assert!(location >= 0, "Uniform not found?!: {}", String::from_utf8_lossy(&namebuf));

		let needs_texture_unit = matches!(ty, gl::SAMPLER_2D | gl::SAMPLER_2D_ARRAY | gl::SAMPLER_1D | gl::SAMPLER_1D_ARRAY | gl::SAMPLER_CUBE | gl::SAMPLER_3D);
		let texture_unit = if needs_texture_unit { texture_slot += 1; texture_slot } else { -1 };
		let namelen = namelen as u8;
		uniforms.push(GlActiveUniform { location, size, ty, texture_unit, namelen, namebuf });
		// println!("Uniform: {} (location: {})", shader.uniforms.last().unwrap().name(), location);
	}

	let id = this.shaders.insert(name, GlShader { program, attribs, uniforms });
	Ok(id)
}

pub fn delete(this: &mut GlGraphics, id: crate::Shader) -> Result<(), crate::GfxError> {
	let Some(shader) = this.shaders.remove(id) else { return Err(crate::GfxError::InvalidHandle) };
	gl_check!(gl::DeleteProgram(shader.program));
	Ok(())
}
