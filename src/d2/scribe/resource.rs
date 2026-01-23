use super::*;

/// Resources required for rendering text.
#[derive(Clone, Debug, Default)]
pub struct FontResource<F> {
	/// The font to use.
	pub font: F,
	/// The texture containing the font's glyphs.
	pub texture: Texture2D,
	/// The shader to use for rendering text.
	pub shader: ShaderProgram,
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
