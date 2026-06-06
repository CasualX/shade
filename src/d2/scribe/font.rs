use super::*;

/// Target that can receive laid-out text glyph quads.
pub trait ITextTarget {
	fn text_quad(&mut self, vertices: &Sprite<TextVertex>);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cursor {
	pub pos: Vec2f,
	// Use u32::MAX to indicate no previous glyph.
	pub(crate) previous_glyph: u32,
}

#[inline]
#[allow(non_snake_case)]
pub const fn Cursor(pos: Vec2f) -> Cursor {
	Cursor { pos, previous_glyph: u32::MAX }
}

/// Font implementation trait.
pub trait IFont {
	/// Write a span of text to the text buffer.
	fn write_span(&self, cv: Option<&mut (dyn ITextTarget + '_)>, scribe: &Scribe, cursor: &mut Cursor, text: &str);
}

impl<'a, T: ?Sized + IFont> IFont for &'a T {
	#[inline]
	fn write_span(&self, cv: Option<&mut (dyn ITextTarget + '_)>, scribe: &Scribe, cursor: &mut Cursor, text: &str) {
		(**self).write_span(cv, scribe, cursor, text)
	}
}

impl<'a, T: ?Sized + IFont> IFont for &'a mut T {
	#[inline]
	fn write_span(&self, cv: Option<&mut (dyn ITextTarget + '_)>, scribe: &Scribe, cursor: &mut Cursor, text: &str) {
		(**self).write_span(cv, scribe, cursor, text)
	}
}

impl<T: IFont> IFont for Option<T> {
	#[inline]
	fn write_span(&self, cv: Option<&mut (dyn ITextTarget + '_)>, scribe: &Scribe, cursor: &mut Cursor, text: &str) {
		let Some(font) = self else { return };
		font.write_span(cv, scribe, cursor, text)
	}
}

impl<T: IFont, E> IFont for Result<T, E> {
	#[inline]
	fn write_span(&self, cv: Option<&mut (dyn ITextTarget + '_)>, scribe: &Scribe, cursor: &mut Cursor, text: &str) {
		let Ok(font) = self else { return };
		font.write_span(cv, scribe, cursor, text)
	}
}
