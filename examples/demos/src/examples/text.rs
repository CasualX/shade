use crate::*;

const PANEL_SOURCE_X: [d2::layout::Unit; 3] = [
	d2::layout::Unit::Abs(12.0),
	d2::layout::Unit::Fr(16.0),
	d2::layout::Unit::Abs(12.0),
];
const PANEL_SOURCE_Y: [d2::layout::Unit; 3] = PANEL_SOURCE_X;
const PANEL_TARGET_X: [d2::layout::Unit; 3] = [
	d2::layout::Unit::Abs(28.0),
	d2::layout::Unit::Fr(1.0),
	d2::layout::Unit::Abs(28.0),
];
const PANEL_TARGET_Y: [d2::layout::Unit; 3] = PANEL_TARGET_X;

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Text::new(g, assets))
}

struct Text {
	font: d2::FontResource<shade::atlas::Font>,
	panel: shade::atlas::Frame,
}

impl Text {
	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Text {
		let (atlas, font) = load_atlas_font(g, assets, "font/atlas.json", "font/font.png", "font", false);
		let panel = atlas.sprites["panel.rounded"].as_frame().expect("panel.rounded must be a single frame").clone();
		Text { font, panel }
	}

	fn draw_panel<'a>(&'a self, cv: &mut d2::TextBuffer<'a>, rect: Bounds2f) {
		let frame = &self.panel;
		let atlas_width = self.font.font.meta.width as f32;
		let atlas_height = self.font.font.meta.height as f32;
		let left = frame.rect.left() as f32;
		let top = frame.rect.top() as f32;
		let right = frame.rect.right() as f32;
		let bottom = frame.rect.bottom() as f32;
		let mut uv_x: [f32; 4] = flex_panel(left, right, &PANEL_SOURCE_X);
		let mut uv_y: [f32; 4] = flex_panel(top, bottom, &PANEL_SOURCE_Y);
		let pos_x: [f32; 4] = flex_panel(rect.left(), rect.right(), &PANEL_TARGET_X);
		let pos_y: [f32; 4] = flex_panel(rect.top(), rect.bottom(), &PANEL_TARGET_Y);
		for x in &mut uv_x {
			*x /= atlas_width;
		}
		for y in &mut uv_y {
			*y /= atlas_height;
		}
		let template = d2::TextTemplate {
			uv: Vec2::ZERO,
			color: Vec4(34, 42, 62, 128),
			outline: Vec4(242, 200, 116, 255),
		};

		cv.uniform.texture = &*self.font.texture;
		cv.shader = Some(&*self.font.shader);
		cv.panel_rect(&template, &uv_x, &uv_y, &pos_x, &pos_y);
	}
}

fn flex_panel<const N: usize, const M: usize>(min: f32, max: f32, template: &[d2::layout::Unit; N]) -> [f32; M] {
	debug_assert_eq!(M, N + 1);
	let spans = d2::layout::flex1d(min, max, None, d2::layout::Justify::Start, template);
	let mut pos = [0.0; M];
	pos[0] = min;
	for i in 0..N {
		if i + 1 < M {
			pos[i + 1] = spans[i][1];
		}
	}
	pos
}

impl DemoInterface for Text {
	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.4, 0.4, 0.7, 1.0), depth: 1.0);

		let mut cv = d2::TextBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.uniform.transform = Transform2::ortho(viewport.cast());
		cv.uniform.unit_range = Vec2::dup(self.font.font.meta.distance_range) / Vec2(self.font.font.meta.width as f32, self.font.font.meta.height as f32);
		cv.uniform.outline_width_relative = 0.125;

		let panel_width = (viewport.width() as f32 - 80.0).clamp(300.0, 620.0);
		let panel_left = (viewport.width() as f32 - panel_width) * 0.5;

		let hello_top = 32.0;
		let hello_line_height = 64.0 * 1.5;
		let mut cursor = d2::Cursor(Vec2(0.0, hello_top));
		let mut scribe = d2::Scribe {
			font_size: 64.0,
			line_height: hello_line_height,
			x_pos: cursor.pos.x,
			top_skew: 8.0,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);

		cv.text_write(&self.font, &mut scribe, &mut cursor, "Hello, \x1b[font_size=96.0]\x1b[font_width_scale=1.5]\x1b[top_skew=0.0]world!");

		let panel_top = hello_top + hello_line_height + 20.0;
		let panel_rect = Bounds2!(panel_left, panel_top, panel_left + panel_width, panel_top + 176.0);
		let panel_text_rect = Bounds2!(panel_rect.left() + 34.0, panel_rect.top() + 30.0, panel_rect.right() - 34.0, panel_rect.bottom() - 26.0);
		let panel_scribe = d2::Scribe {
			font_size: 30.0,
			line_height: 38.0,
			color: Vec4(238, 244, 255, 255),
			outline: Vec4(5, 8, 16, 255),
			..Default::default()
		};

		scribe.font_size = 32.0;
		scribe.line_height = 32.0;
		scribe.font_width_scale = 1.0;
		scribe.color = Vec4(255, 255, 0, 255);

		let bounds = viewport.cast();
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleCenter, "These\nare\nmultiple\nlines.\n");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleLeft, "[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness");
		cv.text_box(&self.font, &scribe, &bounds, d2::TextAlign::MiddleRight, "↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶\n△▽◁▷\n☐☑☒🗹🗷\n⏰💎🔹⚡⛔🏁");

		self.draw_panel(&mut cv, panel_rect);
		cv.text_box(&self.font, &panel_scribe, &panel_text_rect, d2::TextAlign::MiddleCenter, "MTSDF panel frame\none tiny rounded rect, nine-sliced");

		scribe.top_skew = 8.0;
		let rainbow = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W";
		let (rainbow_width, _) = scribe.measure_text(&self.font.font, rainbow);
		let mut cursor = d2::Cursor(Vec2f((viewport.width() as f32 - rainbow_width) * 0.5, viewport.height() as f32 - scribe.font_size));
		cv.text_write(&self.font, &mut scribe, &mut cursor, rainbow);

		cv.draw(g);
		g.end();
	}
}
