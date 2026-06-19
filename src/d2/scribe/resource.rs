use super::*;

/// Resources required for rendering text.
#[derive(Copy, Clone)]
pub struct FontResource<F, T = Box<dyn Texture2D>, S = Box<dyn ShaderProgram>> {
	/// The font to use.
	pub font: F,
	/// The texture containing the font's glyphs.
	pub texture: T,
	/// The shader to use for rendering text.
	pub shader: S,
}

impl<F: IFont> FontResource<F> {
	#[inline]
	pub fn as_dyn(&self) -> FontResource<&dyn IFont, &dyn Texture2D, &dyn ShaderProgram> {
		FontResource {
			font: &self.font,
			texture: &*self.texture,
			shader: &*self.shader,
		}
	}

	#[inline]
	pub fn as_dyn_mut(&mut self) -> FontResource<&mut dyn IFont, &dyn Texture2D, &dyn ShaderProgram> {
		FontResource {
			font: &mut self.font,
			texture: &*self.texture,
			shader: &*self.shader,
		}
	}
}
