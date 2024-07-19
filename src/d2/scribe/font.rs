use super::*;

/// Font implementation trait.
pub trait IFont {
	fn text_width(&self, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) -> f32;
	fn write_span(&self, cv: &mut TextBuffer, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str);
}

impl<'a, T: ?Sized + IFont> IFont for &'a T {
	#[inline]
	fn text_width(&self, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) -> f32 {
		(**self).text_width(scribe, cursor, text)
	}

	#[inline]
	fn write_span(&self, cv: &mut TextBuffer, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) {
		(**self).write_span(cv, scribe, cursor, text)
	}
}

impl<'a, T: ?Sized + IFont> IFont for &'a mut T {
	#[inline]
	fn text_width(&self, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) -> f32 {
		(**self).text_width(scribe, cursor, text)
	}

	#[inline]
	fn write_span(&self, cv: &mut TextBuffer, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) {
		(**self).write_span(cv, scribe, cursor, text)
	}
}

impl<T: IFont> IFont for Option<T> {
	#[inline]
	fn text_width(&self, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) -> f32 {
		let Some(font) = self else { return 0.0 };
		font.text_width(scribe, cursor, text)
	}

	#[inline]
	fn write_span(&self, cv: &mut TextBuffer, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) {
		let Some(font) = self else { return };
		font.write_span(cv, scribe, cursor, text)
	}
}

impl<T: IFont, E> IFont for Result<T, E> {
	#[inline]
	fn text_width(&self, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) -> f32 {
		let Ok(font) = self else { return 0.0 };
		font.text_width(scribe, cursor, text)
	}

	#[inline]
	fn write_span(&self, cv: &mut TextBuffer, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) {
		let Ok(font) = self else { return };
		font.write_span(cv, scribe, cursor, text)
	}
}
