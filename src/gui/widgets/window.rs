use super::*;

/// Draggable window.
pub struct Window {
	key: SlotKey,
	title: StringProperty,
	header_height: i32,
	drag_offset: Option<cvmath::Vec2i>,
	content: ChildWidget,
}

impl Widget for Window {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, _app: &dyn AppState) -> Option<Cursor> {
		if self.drag_offset.is_some() {
			Some(Cursor::Grabbing)
		}
		else {
			Some(Cursor::Grab)
		}
	}

	fn draggable(&self) -> bool {
		true
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, _app: &mut dyn AppState) {
		if let Some(mouse) = event.mouse() {
			if ctx.target == self.key {
				match mouse.kind {
					MouseEventKind::ButtonDown { button: MouseButton::LEFT } => {
						self.drag_offset = Some(mouse.pointer - ctx.bounds.mins);
						scene.capture_pointer(self.key);
					},
					MouseEventKind::ButtonUp { button: MouseButton::LEFT } => {
						self.drag_offset = None;
						scene.release_pointer();
					},
					_ => {},
				}
			}
			else if matches!(mouse.kind, MouseEventKind::ButtonUp { button: MouseButton::LEFT }) {
				self.drag_offset = None;
				scene.release_pointer();
			}
		}
	}

	fn layout(&mut self, ctx: &DrawContext, resx: &dyn Resources, _scene: &mut Scene, app: &dyn AppState) {
		self.header_height = self.measure_header_height(ctx, resx, app);
		let size = ctx.bounds.size();
		self.content.bounds = cvmath::Bounds2!(0, self.header_height, size.x, size.y);
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) {
		self.draw_chrome(im, ctx, resx, app);
	}

	fn children(&self) -> &[ChildWidget] {
		slice::from_ref(&self.content)
	}
}

const HEADER_PAD_X: i32 = 16;
const HEADER_PAD_Y: i32 = 10;

impl Window {
	fn title_scribe(&self) -> d2::Scribe {
		let mut scribe = d2::Scribe {
			font_size: 18.0,
			line_height: 22.0,
			color: cvmath::Vec4!(rgb(244, 245, 247)),
			outline: cvmath::Vec4!(rgba(18, 20, 24, 0.86)),
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);
		scribe
	}

	fn measure_header_height(&self, ctx: &DrawContext, resx: &dyn Resources, app: &dyn AppState) -> i32 {
		let scribe = self.title_scribe();
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		let (text_height, text_width) = self.title.with(app, |title| {
			let (text_width, text_height) = scribe.measure_text(font.font, title);
			let text_height = text_height.ceil() as i32;
			let text_width = text_width.ceil() as i32;
			(text_height, text_width)
		}).unwrap_or_else(|| {
			let title = "<window>";
			let (text_width, text_height) = scribe.measure_text(font.font, title);
			let text_height = text_height.ceil() as i32;
			let text_width = text_width.ceil() as i32;
			(text_height, text_width)
		});
		let width_adjust = if text_width + HEADER_PAD_X * 2 > ctx.bounds.width() { 4 } else { 0 };
		(text_height + HEADER_PAD_Y * 2 + width_adjust).max(40)
	}

	fn draw_title<'a>(&self, im: &mut im::DrawPool<'a>, ctx: &DrawContext, font: &d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>, rc: cvmath::Bounds2i, app: &dyn AppState) {
		let scribe = self.title_scribe();
		if self.title.with(app, |title| {
			im.draw_text_box(ctx, font, &scribe, &rc, d2::TextAlign::MiddleLeft, title);
		}).is_none() {
			im.draw_text_box(ctx, font, &scribe, &rc, d2::TextAlign::MiddleLeft, "<window>");
		}
	}

	fn draw_chrome<'a>(&self, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) -> i32 {
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let header_height = self.header_height;
		let header = cvmath::Bounds2!(rc.left(), rc.top(), rc.right(), rc.top() + header_height);
		let header_text = cvmath::Bounds2!(
			header.left() + HEADER_PAD_X,
			header.top() + HEADER_PAD_Y,
			header.right() - HEADER_PAD_X,
			header.bottom() - HEADER_PAD_Y,
		);
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, rc, cvmath::Vec4!(rgb(37, 40, 48)), shader);
		im.fill_rect(ctx, header, cvmath::Vec4!(rgb(48, 52, 62)), shader);
		im.fill_edge_rect(ctx, rc, cvmath::Vec4!(rgb(83, 88, 101)), 1.0, shader);
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		self.draw_title(im, ctx, &font, header_text, app);
		header_height
	}
}

impl dto::Window {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let title = From::from(self.title);
		let header_height = 40;
		let content = self.content.construct(scene, ctx);
		let content = ChildWidget::new(cvmath::Bounds2!(0, header_height, 0, 0), content);
		let drag_offset = None;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Window { key, title, header_height, drag_offset, content }
		})
	}
}
