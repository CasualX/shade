//! Bundled shader sources.

pub mod glsl330core;
pub mod glsl300es;

#[derive(Copy, Clone)]
pub struct PostProcessCopyUniforms {
	pub texture: crate::Texture2D,
}

impl crate::UniformVisitor for PostProcessCopyUniforms {
	fn visit(&self, set: &mut dyn crate::UniformSetter) {
		set.sampler2d("u_texture", &[self.texture]);
	}
}

#[derive(Copy, Clone)]
pub struct PostProcessCrtUniforms {
	pub texture: crate::Texture2D,
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

impl Default for PostProcessCrtUniforms {
	fn default() -> Self {
		PostProcessCrtUniforms {
			texture: crate::Texture2D::INVALID,
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

impl crate::UniformVisitor for PostProcessCrtUniforms {
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
