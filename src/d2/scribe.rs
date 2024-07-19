use super::*;

pub(crate) mod escape;
mod font;
mod resource;
mod u;
mod v;

pub use self::font::IFont;
pub use self::resource::FontResource;
pub use self::u::TextUniform;
pub use self::v::TextVertex;

pub type TextBuffer = CommandBuffer<TextVertex, TextUniform>;

/// Box alignment.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum BoxAlign {
	TopLeft = 0,
	TopCenter = 1,
	TopRight = 2,
	MiddleLeft = 4,
	MiddleCenter = 5,
	MiddleRight = 6,
	BottomLeft = 8,
	BottomCenter = 9,
	BottomRight = 10,
}

/// Scribe writes text.
#[derive(Clone, Debug, PartialEq)]
pub struct Scribe {
	/// The vertical size of the font.
	pub font_size: f32,
	/// Resize the width of the font as a scale of the font size.
	pub font_width_scale: f32,
	/// The height of a line of text.
	pub line_height: f32,
	/// The y position of the baseline from the bottom of the line.
	pub baseline: f32,
	/// The x position of the next line.
	pub x_pos: f32,
	/// Additional spacing between letters.
	pub letter_spacing: f32,
	/// Skew the top vertex x positions to simulate faux italics.
	pub top_skew: f32,
	/// The color of the text.
	pub color: Vec4<u8>,
	/// The color of the outline.
	pub outline: Vec4<u8>,
}

impl Default for Scribe {
	#[inline]
	fn default() -> Self {
		Scribe {
			font_size: 16.0,
			font_width_scale: 1.0,
			line_height: 16.0,
			baseline: 0.0,
			x_pos: 0.0,
			letter_spacing: 0.0,
			top_skew: 0.0,
			color: Vec4(255, 255, 255, 255),
			outline: Vec4(0, 0, 0, 255),
		}
	}
}

impl Scribe {
	/// Sets the baseline relative to the line height and font size.
	///
	/// A fraction of `0.0` will place the baseline at the bottom of the line,
	/// `1.0` will place it at the top, and `0.5` will place it in the middle.
	#[inline]
	pub fn set_baseline_relative(&mut self, fraction: f32) {
		self.baseline = (self.line_height - self.font_size) * fraction;
	}

	/// Measures the width of a text string.
	#[inline]
	pub fn text_width(&self, cursor: &mut Vec2<f32>, font: &dyn IFont, text: &str) -> f32 {
		font.text_width(self, cursor, text)
	}
}

#[inline(never)]
fn count_lines_height(s: &str, line_height: f32) -> f32 {
	s.lines().count() as i32 as f32 * line_height
}

impl TextBuffer {
	/// Writes a text string.
	#[inline]
	pub fn text_write(&mut self, font: &FontResource<impl IFont>, scribe: &Scribe, cursor: &mut Vec2<f32>, text: &str) {
		self.shader = font.shader;
		font.font.write_span(self, scribe, cursor, text);
	}

	/// Writes a text string using the box model.
	///
	/// The text will be aligned within the rect according to the alignment.
	pub fn text_box(&mut self, font: &FontResource<impl IFont>, scribe: &Scribe, rect: &cvmath::Rect<f32>, align: BoxAlign, text: &str) {
		self.shader = font.shader;
		let mut y = match align {
			BoxAlign::TopLeft | BoxAlign::TopCenter | BoxAlign::TopRight => rect.mins.y,
			BoxAlign::MiddleLeft | BoxAlign::MiddleCenter | BoxAlign::MiddleRight => rect.mins.y + (rect.height() - count_lines_height(text, scribe.line_height)) * 0.5,
			BoxAlign::BottomLeft | BoxAlign::BottomCenter | BoxAlign::BottomRight => rect.maxs.y - count_lines_height(text, scribe.line_height),
		};

		for line in text.lines() {
			let x = match align {
				BoxAlign::TopLeft | BoxAlign::MiddleLeft | BoxAlign::BottomLeft => rect.mins.x,
				BoxAlign::TopCenter | BoxAlign::MiddleCenter | BoxAlign::BottomCenter => rect.mins.x + (rect.width() - scribe.text_width(&mut {Vec2::ZERO}, &font.font, line)) * 0.5,
				BoxAlign::TopRight | BoxAlign::MiddleRight | BoxAlign::BottomRight => rect.maxs.x - scribe.text_width(&mut {Vec2::ZERO}, &font.font, line),
			};
			self.text_write(font, scribe, &mut Vec2(x, y), line);
			y += scribe.line_height;
		}
	}
}
