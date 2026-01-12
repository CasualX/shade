use super::*;

struct GlTextureFormat {
	internal_format: GLenum,
	format: GLenum,
	type_: GLenum,
	align: GLint,
}
impl GlTextureFormat {
	fn get(format: crate::TextureFormat, config: &GlConfig) -> GlTextureFormat {
		let (internal_format, format, type_, align) = if config.srgb {
			match format {
				crate::TextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT, 4),
				crate::TextureFormat::RGB32F => (gl::RGB32F, gl::RGB, gl::FLOAT, 4),
				crate::TextureFormat::RG32F => (gl::RG32F, gl::RG, gl::FLOAT, 4),
				crate::TextureFormat::R32F => (gl::R32F, gl::RED, gl::FLOAT, 4),
				crate::TextureFormat::SRGBA8 => (gl::SRGB8_ALPHA8, gl::RGBA, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::SRGB8 => (gl::SRGB8, gl::RGB, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::Depth16 => (gl::DEPTH_COMPONENT16, gl::DEPTH_COMPONENT, gl::UNSIGNED_SHORT, 2),
				crate::TextureFormat::Depth24 => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_INT, 4),
				crate::TextureFormat::Depth32F => (gl::DEPTH_COMPONENT32F, gl::DEPTH_COMPONENT, gl::FLOAT, 4),
				crate::TextureFormat::Depth24Stencil8 => (gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 4),
			}
		}
		else {
			match format {
				crate::TextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT, 4),
				crate::TextureFormat::RGB32F => (gl::RGB32F, gl::RGB, gl::FLOAT, 4),
				crate::TextureFormat::RG32F => (gl::RG32F, gl::RG, gl::FLOAT, 4),
				crate::TextureFormat::R32F => (gl::R32F, gl::RED, gl::FLOAT, 4),
				crate::TextureFormat::SRGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::SRGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE, 1),
				crate::TextureFormat::Depth16 => (gl::DEPTH_COMPONENT16, gl::DEPTH_COMPONENT, gl::UNSIGNED_SHORT, 2),
				crate::TextureFormat::Depth24 => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_INT, 4),
				crate::TextureFormat::Depth32F => (gl::DEPTH_COMPONENT32F, gl::DEPTH_COMPONENT, gl::FLOAT, 4),
				crate::TextureFormat::Depth24Stencil8 => (gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 4),
			}
		};
		GlTextureFormat { internal_format, format, type_, align }
	}
}


pub fn create(this: &mut GlGraphics, name: Option<&str>, info: &crate::Texture2DInfo) -> crate::Texture2D {
	let mut texture = 0;
	gl_check!(gl::GenTextures(1, &mut texture));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)));
	if let Some(compare) = info.props.compare {
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE as GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC, gl_depth_func(compare) as GLint));
	}
	else {
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE, gl::NONE as GLint));
	}
	gl_check!(gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, info.props.border_color.as_ptr()));
	// Allocate texture storage (required for framebuffer attachments)
	let GlTextureFormat { internal_format, .. } = GlTextureFormat::get(info.format, &this.config);
	gl_check!(gl::TexStorage2D(gl::TEXTURE_2D, info.props.mip_levels as GLsizei, internal_format, info.width, info.height));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
	let id = this.textures.textures2d.insert(name, GlTexture2D { texture, info: *info });
	return id;
}

pub fn find(this: &GlGraphics, name: &str) -> crate::Texture2D {
	this.textures.textures2d.find_id(name).unwrap_or(crate::Texture2D::INVALID)
}

pub fn get_info(this: &GlGraphics, id: crate::Texture2D) -> Option<&crate::Texture2DInfo> {
	this.textures.textures2d.get(id).map(|texture| &texture.info)
}

pub fn generate_mipmap(this: &mut GlGraphics, id: crate::Texture2D) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
	gl_check!(gl::GenerateMipmap(gl::TEXTURE_2D));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

pub fn update(this: &mut GlGraphics, id: crate::Texture2D, info: &crate::Texture2DInfo) -> crate::Texture2D {
	let Some(texture) = this.textures.textures2d.get_mut(id) else {
		return create(this, None, info);
	};

	// Short-circuit if no changes.
	if &texture.info == info {
		return id;
	}

	// With TexStorage2D, the texture storage is immutable. Any change to
	// format/size/mip count requires creating a new GL texture object.
	let realloc = texture.info.width != info.width
		|| texture.info.height != info.height
		|| texture.info.format != info.format
		|| texture.info.props.mip_levels != info.props.mip_levels;

	if realloc {
		gl_check!(gl::DeleteTextures(1, &texture.texture));
		gl_check!(gl::GenTextures(1, &mut texture.texture));
	}

	gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
	if realloc {
		let GlTextureFormat { internal_format, .. } = GlTextureFormat::get(info.format, &this.config);
		gl_check!(gl::TexStorage2D(gl::TEXTURE_2D, info.props.mip_levels as GLsizei, internal_format, info.width, info.height));
	}

	// (Re)apply sampler state.
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl_texture_wrap(info.props.wrap_u)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl_texture_wrap(info.props.wrap_v)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_texture_filter_mag(info.props.filter_mag)));
	gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_texture_filter_min(&info.props)));
	if let Some(compare) = info.props.compare {
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE as GLint));
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC, gl_depth_func(compare) as GLint));
	}
	else {
		gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE, gl::NONE as GLint));
	}
	gl_check!(gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, info.props.border_color.as_ptr()));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));

	texture.info = *info;
	return id;
}

pub fn write(this: &mut GlGraphics, id: crate::Texture2D, level: u8, data: &[u8]) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	assert!(level < texture.info.props.mip_levels, "Invalid mip level {}", level);
	assert!(texture.info.props.usage.has(crate::TextureUsage::WRITE), "Texture was not created with WRITE usage");
	this.metrics.bytes_uploaded = usize::wrapping_add(this.metrics.bytes_uploaded, data.len());
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
	let GlTextureFormat { format, type_, align, .. } = GlTextureFormat::get(texture.info.format, &this.config);
	let (w, h, expected_size) = texture.info.mip_size(level);
	assert_eq!(data.len(), expected_size, "Data size does not match texture mip dimensions");
	gl_check!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, align)); // Force correct byte alignment
	gl_check!(gl::TexSubImage2D(gl::TEXTURE_2D, level as GLint, 0, 0, w, h, format, type_, data.as_ptr() as *const _));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

pub fn read_into(this: &mut GlGraphics, id: crate::Texture2D, level: u8, data: &mut [u8]) {
	let Some(texture) = this.textures.textures2d.get(id) else { return };
	assert!(level < texture.info.props.mip_levels, "Invalid mip level {}", level);
	assert!(texture.info.props.usage.has(crate::TextureUsage::READBACK), "Texture was not created with READBACK usage");
	this.metrics.bytes_downloaded = usize::wrapping_add(this.metrics.bytes_downloaded, data.len());
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, texture.texture));
	let GlTextureFormat { format, type_, align, .. } = GlTextureFormat::get(texture.info.format, &this.config);
	let (_w, _h, expected_size) = texture.info.mip_size(level);
	assert_eq!(data.len(), expected_size, "Data size does not match texture mip dimensions");
	gl_check!(gl::PixelStorei(gl::PACK_ALIGNMENT, align)); // Force correct byte alignment
	gl_check!(gl::GetTexImage(gl::TEXTURE_2D, level as GLint, format, type_, data.as_mut_ptr() as *mut _));
	gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
}

pub fn free(this: &mut GlGraphics, id: crate::Texture2D, mode: crate::FreeMode) {
	assert_eq!(mode, crate::FreeMode::Delete, "Only FreeMode::Delete is implemented");
	let Some(texture) = this.textures.textures2d.remove(id) else { return };
	gl_check!(gl::DeleteTextures(1, &texture.texture));
}
