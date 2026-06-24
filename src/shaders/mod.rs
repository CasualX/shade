//! Bundled shader sources.

pub const COLOR: &str = include_str!("color.glsl");
pub const GRADIENT: &str = include_str!("gradient.glsl");
pub const TEXTURED: &str = include_str!("textured.glsl");
pub const PIXELART: &str = include_str!("pixelart.glsl");
pub const MTSDF: &str = include_str!("mtsdf.glsl");
pub const COLOR3D: &str = include_str!("color3d.glsl");
pub const POST_PROCESS_COPY: &str = include_str!("post_process.copy.glsl");
pub const POST_PROCESS_CRT: &str = include_str!("post_process.crt.glsl");
pub const POST_PROCESS_PIXELART: &str = include_str!("post_process.pixelart.glsl");
pub const POST_PROCESS_MELT: &str = include_str!("post_process.melt.glsl");

#[derive(Copy, Clone)]
pub struct PostProcessCopyUniforms<'a> {
	pub texture: &'a dyn crate::Texture2D,
}

impl<'a> crate::UniformVisitor for PostProcessCopyUniforms<'a> {
	fn visit(&self, set: &mut dyn crate::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
	}
}

unsafe impl<'a> crate::TUniformKey for PostProcessCopyUniforms<'a> {
	#[inline]
	fn key() -> std::any::TypeId {
		std::any::TypeId::of::<PostProcessCopyUniforms<'static>>()
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PostProcessCrtUniforms<'a> {
	pub texture: &'a dyn crate::Texture2D,
	pub scanline_intensity: f32,
	pub scanline_count: f32,
	pub adaptive_intensity: f32,
	pub brightness: f32,
	pub contrast: f32,
	pub saturation: f32,
	pub bloom_intensity: f32,
	pub bloom_threshold: f32,
	pub rgb_shift: f32,
	pub vignette_strength: f32,
	pub curvature: f32,
	pub flicker_strength: f32,
	pub time: f32,
}

impl<'a> Default for PostProcessCrtUniforms<'a> {
	fn default() -> Self {
		PostProcessCrtUniforms {
			texture: &crate::DefaultTexture2D,
			scanline_intensity: 0.5,
			scanline_count: 256.0,
			adaptive_intensity: 0.3,
			brightness: 1.5,
			contrast: 1.05,
			saturation: 1.1,
			bloom_intensity: 0.5,
			bloom_threshold: 0.5,
			rgb_shift: 0.005,
			vignette_strength: 0.3,
			curvature: 0.3,
			flicker_strength: 0.02,
			time: 0.0,
		}
	}
}

impl<'a> crate::UniformVisitor for PostProcessCrtUniforms<'a> {
	fn visit(&self, set: &mut dyn crate::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
		set.value("u_scanline_intensity", &self.scanline_intensity);
		set.value("u_scanline_count", &self.scanline_count);
		set.value("u_adaptive_intensity", &self.adaptive_intensity);
		set.value("u_brightness", &self.brightness);
		set.value("u_contrast", &self.contrast);
		set.value("u_saturation", &self.saturation);
		set.value("u_bloom_intensity", &self.bloom_intensity);
		set.value("u_bloom_threshold", &self.bloom_threshold);
		set.value("u_rgb_shift", &self.rgb_shift);
		set.value("u_vignette_strength", &self.vignette_strength);
		set.value("u_curvature", &self.curvature);
		set.value("u_flicker_strength", &self.flicker_strength);
		set.value("u_time", &self.time);
	}
}

unsafe impl<'a> crate::TUniformKey for PostProcessCrtUniforms<'a> {
	#[inline]
	fn key() -> std::any::TypeId {
		std::any::TypeId::of::<PostProcessCrtUniforms<'static>>()
	}
}
