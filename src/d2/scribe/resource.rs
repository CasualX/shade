use super::*;

#[derive(Clone, Debug, Default)]
pub struct FontResource<F: IFont> {
	pub font: F,
	pub texture: Texture2D,
	pub shader: Shader,
}

impl<F: IFont> FontResource<F> {
	#[inline]
	pub fn as_dyn(&self) -> FontResource<&dyn IFont> {
		FontResource {
			font: &self.font,
			texture: self.texture,
			shader: self.shader,
		}
	}
	#[inline]
	pub fn as_dyn_mut(&mut self) -> FontResource<&mut dyn IFont> {
		FontResource {
			font: &mut self.font,
			texture: self.texture,
			shader: self.shader,
		}
	}
}
