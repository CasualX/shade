use super::*;

const DEFAULT_FONT_SIZE: f32 = 15.0;
const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.22;
const DEFAULT_TEXT_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const DEFAULT_OUTLINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(18, 20, 24, 0.86));
const DEFAULT_ALIGN: d2::TextAlign = d2::TextAlign::MiddleLeft;

/// Static text label widget.
pub struct Label {
	key: SlotKey,
	text: StringProperty,
	font_size: f32,
	color: cvmath::Vec4<u8>,
	align: d2::TextAlign,
}

impl Widget for Label {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn hittable(&self) -> bool {
		false
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState, app_ctx: &dyn AppContext) {
		let mut scribe = d2::Scribe {
			font_size: self.font_size,
			line_height: self.font_size * DEFAULT_LINE_HEIGHT_SCALE,
			color: self.color,
			outline: DEFAULT_OUTLINE_COLOR,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		if self.text.with(app, app_ctx, |text| {
			im.draw_text_box(ctx, &font, &scribe, &rc, self.align, text);
		}).is_none() {
			im.draw_text_box(ctx, &font, &scribe, &rc, self.align, "<label>");
		}
	}
}

impl dto::Label {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let text = From::from(self.text);
		let font_size = self.font_size.unwrap_or(DEFAULT_FONT_SIZE);
		let color = self.color.unwrap_or(DEFAULT_TEXT_COLOR);
		let align = self.align.unwrap_or(DEFAULT_ALIGN);
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Label { key, text, font_size, color, align }
		})
	}
}
