use crate::*;

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Text::new(g, assets))
}

struct Text {
	font: d2::FontResource<shade::msdfgen::Font>,
}

impl Text {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Text {
		let font = load_font(g, assets, false);
		Text { font }
	}
}

impl DemoInterface for Text {
	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.4, 0.4, 0.7, 1.0), depth: 1.0);

		let mut cv = d2::TextBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.uniform.transform = Transform2::ortho(viewport.cast());
		cv.uniform.outline_width_relative = 0.125;

		let mut pos = Vec2(0.0, 0.0);
		let mut scribe = d2::Scribe {
			font_size: 64.0,
			line_height: 64.0 * 1.5,
			x_pos: pos.x,
			top_skew: 8.0,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);

		cv.text_write(
			&self.font,
			&mut scribe,
			&mut pos,
			"Hello, \x1b[font_size=96.0]\x1b[font_width_scale=1.5]\x1b[top_skew=0.0]world!",
		);

		scribe.font_size = 32.0;
		scribe.line_height = 32.0;
		scribe.font_width_scale = 1.0;
		scribe.color = Vec4(255, 255, 0, 255);

		let bounds = viewport.cast();
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleCenter, "These\nare\nmultiple\nlines.\n");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleLeft, "[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleRight, "↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶\n△▽◁▷\n☐☑☒🗹🗷\n⏰💎🔹⚡⛔🏁");

		scribe.top_skew = 8.0;
		let rainbow = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W";
		let rainbow_width = scribe.text_width(&mut { Vec2::ZERO }, &self.font.font, rainbow);
		let mut pos = Vec2f((viewport.width() as f32 - rainbow_width) * 0.5, viewport.height() as f32 - scribe.font_size);
		cv.text_write(&self.font, &mut scribe, &mut pos, rainbow);

		cv.draw(g);
		g.end();
	}
}
