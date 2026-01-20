use super::*;

struct WebGLTextureFormat {
	internalformat: GLenum,
	format: GLenum,
	type_: GLenum,
	align: GLint,
}

impl WebGLTextureFormat {
	fn get(format: crate::TextureFormat) -> WebGLTextureFormat {
		let (internalformat, format, type_, align) = match format {
			crate::TextureFormat::RGBA8 => (api::RGBA8, api::RGBA, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::RGB8 => (api::RGB8, api::RGB, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::RG8 => (api::RG8, api::RG, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::R8 => (api::R8, api::RED, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::RGBA32F => (api::RGBA32F, api::RGBA, api::FLOAT, 4),
			crate::TextureFormat::RGB32F => (api::RGB32F, api::RGB, api::FLOAT, 4),
			crate::TextureFormat::RG32F => (api::RG32F, api::RG, api::FLOAT, 4),
			crate::TextureFormat::R32F => (api::R32F, api::RED, api::FLOAT, 4),
			crate::TextureFormat::SRGBA8 => (api::SRGB8_ALPHA8, api::RGBA, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::SRGB8 => (api::SRGB8, api::RGB, api::UNSIGNED_BYTE, 1),
			crate::TextureFormat::Depth16 => (api::DEPTH_COMPONENT16, api::DEPTH_COMPONENT, api::UNSIGNED_SHORT, 2),
			crate::TextureFormat::Depth24 => (api::DEPTH_COMPONENT24, api::DEPTH_COMPONENT, api::UNSIGNED_INT, 4),
			crate::TextureFormat::Depth32F => (api::DEPTH_COMPONENT32F, api::DEPTH_COMPONENT, api::FLOAT, 4),
			crate::TextureFormat::Depth24Stencil8 => (api::DEPTH24_STENCIL8, api::DEPTH_STENCIL, api::UNSIGNED_INT_24_8, 4),
		};
		WebGLTextureFormat { internalformat, format, type_, align }
	}
}

pub fn create(this: &mut WebGLGraphics, name: Option<&str>, info: &crate::Texture2DInfo) -> crate::Texture2D {
	let texture = unsafe { api::createTexture() };
	unsafe { api::bindTexture(api::TEXTURE_2D, texture) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)) };
	if let Some(compare) = info.props.compare {
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_MODE, api::COMPARE_REF_TO_TEXTURE as GLint) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_FUNC, gl_depth_func(compare) as GLint) };
	}
	else {
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_MODE, api::NONE as GLint) };
	}
	let WebGLTextureFormat { internalformat, .. } = WebGLTextureFormat::get(info.format);
	unsafe { api::texStorage2D(api::TEXTURE_2D, info.props.mip_levels as GLsizei, internalformat, info.width, info.height) };
	unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
	let id = this.textures.textures2d.insert(name, WebGLTexture2D { texture, info: *info });
	return id;
}

pub fn find(this: &WebGLGraphics, name: &str) -> crate::Texture2D {
	this.textures.textures2d.find_id(name).unwrap_or(crate::Texture2D::INVALID)
}

pub fn get_info(this: &WebGLGraphics, id: crate::Texture2D) -> Option<&crate::Texture2DInfo> {
	this.textures.textures2d.get(id).map(|texture| &texture.info)
}

pub fn generate_mipmap(this: &mut WebGLGraphics, id: crate::Texture2D) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
	unsafe { api::generateMipmap(api::TEXTURE_2D) };
	unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
}

pub fn update(this: &mut WebGLGraphics, id: crate::Texture2D, info: &crate::Texture2DInfo) -> crate::Texture2D {
	let Some(texture) = this.textures.textures2d.get_mut(id) else {
		return create(this, None, info);
	};

	// Short-circuit if no changes.
	if &texture.info == info {
		return id;
	}

	let realloc = texture.info.width != info.width ||
		texture.info.height != info.height ||
		texture.info.format != info.format ||
		texture.info.props.mip_levels != info.props.mip_levels;

	// With TexStorage2D, the texture storage is immutable. Any change to
	// format/size/mip count requires creating a new GL texture object.
	if realloc {
		unsafe { api::deleteTexture(texture.texture) };
		texture.texture = unsafe { api::createTexture() };
	}

	unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
	if realloc {
		let WebGLTextureFormat { internalformat, .. } = WebGLTextureFormat::get(info.format);
		unsafe { api::texStorage2D(api::TEXTURE_2D, info.props.mip_levels as GLsizei, internalformat, info.width, info.height) };
	}
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)) };
	unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)) };
	if let Some(compare) = info.props.compare {
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_MODE, api::COMPARE_REF_TO_TEXTURE as GLint) };
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_FUNC, gl_depth_func(compare) as GLint) };
	}
	else {
		unsafe { api::texParameteri(api::TEXTURE_2D, api::TEXTURE_COMPARE_MODE, api::NONE as GLint) };
	}

	unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
	texture.info = *info;
	return id;
}

pub fn write(this: &mut WebGLGraphics, id: crate::Texture2D, level: u8, data: &[u8]) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	assert!(level < texture.info.props.mip_levels, "Invalid mip level {}", level);
	assert!(texture.info.props.usage.has(crate::TextureUsage::WRITE), "Texture was not created with WRITE usage");
	this.metrics.bytes_uploaded = usize::wrapping_add(this.metrics.bytes_uploaded, data.len());
	unsafe { api::bindTexture(api::TEXTURE_2D, texture.texture) };
	let WebGLTextureFormat { format, type_, align, .. } = WebGLTextureFormat::get(texture.info.format);
	let (w, h, expected_size) = texture.info.mip_size(level);
	assert_eq!(data.len(), expected_size, "Data size does not match texture mip dimensions");
	unsafe { api::pixelStorei(api::UNPACK_ALIGNMENT, align) };
	unsafe { api::texSubImage2D(api::TEXTURE_2D, level as GLint, 0, 0, w, h, format, type_, data.as_ptr(), data.len()) };
	unsafe { api::bindTexture(api::TEXTURE_2D, 0) };
}

pub fn read_into(this: &mut WebGLGraphics, id: crate::Texture2D, level: u8, data: &mut [u8]) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	assert!(level < texture.info.props.mip_levels, "Invalid mip level {}", level);
	assert!(texture.info.props.usage.has(crate::TextureUsage::READBACK), "Texture was not created with READBACK usage");
	this.metrics.bytes_downloaded = usize::wrapping_add(this.metrics.bytes_downloaded, data.len());

	let fbo = unsafe { api::createFramebuffer() };
	unsafe { api::bindFramebuffer(api::FRAMEBUFFER, fbo) };

	let is_depth = texture.info.format.is_depth();
	if is_depth {
		if texture.info.format == crate::TextureFormat::Depth24Stencil8 {
			unsafe { api::framebufferTexture2D(api::FRAMEBUFFER, api::DEPTH_STENCIL_ATTACHMENT, api::TEXTURE_2D, texture.texture, level as GLint) };
		}
		else {
			unsafe { api::framebufferTexture2D(api::FRAMEBUFFER, api::DEPTH_ATTACHMENT, api::TEXTURE_2D, texture.texture, level as GLint) };
		}
		let draw_none = [api::NONE as GLenum];
		unsafe { api::drawBuffers(draw_none.len() as i32, draw_none.as_ptr()) };
		unsafe { api::readBuffer(api::NONE) };
	}
	else {
		unsafe { api::framebufferTexture2D(api::FRAMEBUFFER, api::COLOR_ATTACHMENT0, api::TEXTURE_2D, texture.texture, level as GLint) };
		let draw_buffers = [api::COLOR_ATTACHMENT0 as GLenum];
		unsafe { api::drawBuffers(draw_buffers.len() as i32, draw_buffers.as_ptr()) };
		unsafe { api::readBuffer(api::COLOR_ATTACHMENT0) };
	}

	#[cfg(debug_assertions)] {
		let status = unsafe { api::checkFramebufferStatus(api::FRAMEBUFFER) };
		if status != api::FRAMEBUFFER_COMPLETE {
			panic!("Readback framebuffer is incomplete: status = 0x{:X}", status);
		}
	}

	let WebGLTextureFormat { format, type_, align, .. } = WebGLTextureFormat::get(texture.info.format);
	let (w, h, expected_size) = texture.info.mip_size(level);
	assert_eq!(data.len(), expected_size, "Data size does not match texture mip dimensions");
	unsafe { api::pixelStorei(api::PACK_ALIGNMENT, align) };
	unsafe { api::readPixels(0, 0, w, h, format, type_, data.as_mut_ptr(), data.len()) };

	unsafe { api::bindFramebuffer(api::FRAMEBUFFER, 0) };
	unsafe { api::deleteFramebuffer(fbo) };
}

pub fn free(this: &mut WebGLGraphics, id: crate::Texture2D, mode: crate::FreeMode) {
	assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
	let Some(texture) = this.textures.textures2d.remove(id) else { return };
	unsafe { api::deleteTexture(texture.texture) };
}
