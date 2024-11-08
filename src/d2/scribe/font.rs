use super::*;

/// Font implementation trait.
pub trait IFont {
	/// Write a span of text to the text buffer.
	fn write_span(&self, cv: Option<&mut TextBuffer>, scribe_st: &mut Scribe, cursor: &mut Vec2<f32>, text: &str);
}

impl<'a, T: ?Sized + IFont> IFont for &'a T {
	#[inline]
	fn write_span(&self, cv: Option<&mut TextBuffer>, scribe_st: &mut Scribe, cursor: &mut Vec2<f32>, text: &str) {
		(**self).write_span(cv, scribe_st, cursor, text)
	}
}

impl<'a, T: ?Sized + IFont> IFont for &'a mut T {
	#[inline]
	fn write_span(&self, cv: Option<&mut TextBuffer>, scribe_st: &mut Scribe, cursor: &mut Vec2<f32>, text: &str) {
		(**self).write_span(cv, scribe_st, cursor, text)
	}
}

impl<T: IFont> IFont for Option<T> {
	#[inline]
	fn write_span(&self, cv: Option<&mut TextBuffer>, scribe_st: &mut Scribe, cursor: &mut Vec2<f32>, text: &str) {
		let Some(font) = self else { return };
		font.write_span(cv, scribe_st, cursor, text)
	}
}

impl<T: IFont, E> IFont for Result<T, E> {
	#[inline]
	fn write_span(&self, cv: Option<&mut TextBuffer>, scribe_st: &mut Scribe, cursor: &mut Vec2<f32>, text: &str) {
		let Ok(font) = self else { return };
		font.write_span(cv, scribe_st, cursor, text)
	}
}
