use super::*;

pub(crate) mod escape;
mod font;
mod resource;
mod v;

pub use self::font::IFont;
pub use self::resource::FontResource;
pub use self::v::*;

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
	pub fn text_width(&self, cursor: &mut Vec2f, font: &dyn IFont, text: &str) -> f32 {
		let mut scribe_st = self.clone();
		let mut width = 0.0;
		let mut x_pos = cursor.x;
		for line in text.lines() {
			font.write_span(None, &mut scribe_st, cursor, line);
			width = f32::max(width, cursor.x - x_pos);
			x_pos = scribe_st.x_pos;
			cursor.x = scribe_st.x_pos;
			cursor.y += scribe_st.line_height;
		}
		return width;
	}

	/// Measures the height of a text string.
	#[inline(never)]
	pub fn text_height(&self, text: &str) -> f32 {
		text.lines().count() as i32 as f32 * self.line_height
	}
}

impl TextBuffer {
	/// Writes a text string.
	///
	/// Escape sequences can modify the scribe properties in the middle of the text string,
	/// strip user controlled text of ascii escape characters to avoid this.
	pub fn text_write<T: fmt::Display>(&mut self, font: &FontResource<impl IFont>, scribe: &mut Scribe, cursor: &mut Vec2f, text: T) {
		self.shader = font.shader;
		text_write(self, font.as_dyn().font, scribe, cursor, &text);
	}

	/// Writes individual lines of text strings using the box model.
	///
	/// The text will be aligned within the rect according to the alignment.
	///
	/// Escape sequences can modify the scribe properties in the middle of the text string,
	/// strip user controlled text of ascii escape characters to avoid this.
	pub fn text_lines(&mut self, font: &FontResource<impl IFont>, scribe: &Scribe, rect: &Bounds2<f32>, align: BoxAlign, lines: &[&dyn fmt::Display]) {
		self.shader = font.shader;
		text_lines(self, font.as_dyn().font, scribe, rect, align, lines);
	}

	/// Writes a text string using the box model.
	///
	/// The text will be aligned within the rect according to the alignment.
	///
	/// Escape sequences can modify the scribe properties in the middle of the text string,
	/// strip user controlled text of ascii escape characters to avoid this.
	pub fn text_box(&mut self, font: &FontResource<impl IFont>, scribe: &Scribe, rect: &Bounds2<f32>, align: BoxAlign, text: &str) {
		self.shader = font.shader;
		text_box(self, font.as_dyn().font, scribe, rect, align, text);
	}
}

#[repr(transparent)]
struct FormatFn<F>(F);

impl<F: FnMut(&str) -> fmt::Result> fmt::Write for FormatFn<F> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.0(s)
	}
}

fn text_write(buf: &mut TextBuffer, font: &dyn IFont, scribe: &mut Scribe, cursor: &mut Vec2f, text: &dyn fmt::Display) {
	let mut writer = FormatFn(move |text: &str| {
		font.write_span(Some(buf), scribe, cursor, text);
		Ok(())
	});
	let _ = fmt::write(&mut writer, format_args!("{}", text));
}

fn text_lines(buf: &mut TextBuffer, font: &dyn IFont, scribe: &Scribe, rect: &Bounds2<f32>, align: BoxAlign, lines: &[&dyn fmt::Display]) {
	let height = lines.len() as isize as i32 as f32 * scribe.line_height;

	let mut y = match align {
		BoxAlign::TopLeft | BoxAlign::TopCenter | BoxAlign::TopRight => rect.mins.y,
		BoxAlign::MiddleLeft | BoxAlign::MiddleCenter | BoxAlign::MiddleRight => rect.mins.y + (rect.height() - height) * 0.5,
		BoxAlign::BottomLeft | BoxAlign::BottomCenter | BoxAlign::BottomRight => rect.maxs.y - height,
	};

	let mut scribe = scribe.clone();
	for &args in lines {
		let text_width = |args| {
			let mut width = 0.0;
			let mut width_calc = FormatFn(|text: &str| {
				width += scribe.text_width(&mut {Vec2::ZERO}, font, text);
				Ok(())
			});
			let _ = fmt::write(&mut width_calc, format_args!("{}", args));
			width
		};

		let x = match align {
			BoxAlign::TopLeft | BoxAlign::MiddleLeft | BoxAlign::BottomLeft => rect.mins.x,
			BoxAlign::TopCenter | BoxAlign::MiddleCenter | BoxAlign::BottomCenter => rect.mins.x + (rect.width() - text_width(args)) * 0.5,
			BoxAlign::TopRight | BoxAlign::MiddleRight | BoxAlign::BottomRight => rect.maxs.x - text_width(args),
		};

		let mut cursor = Vec2(x, y);
		let mut writer = FormatFn(|text: &str| {
			font.write_span(Some(buf), &mut scribe, &mut cursor, text);
			Ok(())
		});
		let _ = fmt::write(&mut writer, format_args!("{}", args));

		y += scribe.line_height;
	}
}

fn text_box(buf: &mut TextBuffer, font: &dyn IFont, scribe: &Scribe, rect: &Bounds2<f32>, align: BoxAlign, text: &str) {
	let mut y = match align {
		BoxAlign::TopLeft | BoxAlign::TopCenter | BoxAlign::TopRight => rect.mins.y,
		BoxAlign::MiddleLeft | BoxAlign::MiddleCenter | BoxAlign::MiddleRight => rect.mins.y + (rect.height() - scribe.text_height(text)) * 0.5,
		BoxAlign::BottomLeft | BoxAlign::BottomCenter | BoxAlign::BottomRight => rect.maxs.y - scribe.text_height(text),
	};

	let mut scribe = scribe.clone();
	for line in text.lines() {
		let x = match align {
			BoxAlign::TopLeft | BoxAlign::MiddleLeft | BoxAlign::BottomLeft => rect.mins.x,
			BoxAlign::TopCenter | BoxAlign::MiddleCenter | BoxAlign::BottomCenter => rect.mins.x + (rect.width() - scribe.text_width(&mut {Vec2::ZERO}, font, line)) * 0.5,
			BoxAlign::TopRight | BoxAlign::MiddleRight | BoxAlign::BottomRight => rect.maxs.x - scribe.text_width(&mut {Vec2::ZERO}, font, line),
		};
		font.write_span(Some(buf), &mut scribe, &mut Vec2(x, y), line);
		y += scribe.line_height;
	}
}
