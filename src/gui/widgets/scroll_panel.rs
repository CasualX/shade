use super::*;

/// Vertically scrollable container.
pub struct ScrollPanel {
	key: SlotKey,
	content: ChildWidget,
	content_height: i32,
	vbar: ScrollVBar,
}

impl ScrollPanel {
	fn update_content_bounds(&mut self, viewport_size: cvmath::Vec2i) {
		let content_width = self.content_width(viewport_size);
		let scroll_y = self.vbar.scroll_y(viewport_size.y, self.content_height);
		self.content.bounds = cvmath::Bounds2!(0, -scroll_y, content_width, self.content_height - scroll_y);
	}

	fn content_width(&self, viewport_size: cvmath::Vec2i) -> i32 {
		(viewport_size.x - self.vbar.reserved_width(viewport_size, self.content_height)).max(0)
	}
}

impl Widget for ScrollPanel {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, _app: &dyn AppState, _app_ctx: &dyn AppContext) -> Option<Cursor> {
		self.vbar.cursor()
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, _app: &mut dyn AppState, _app_ctx: &mut dyn AppContext) {
		let Some(mouse) = event.mouse() else {
			return;
		};

		self.vbar.event(mouse, ctx, self.key, self.content_height, scene);
	}

	fn layout(&mut self, ctx: &DrawContext, _resx: &dyn Resources, _scene: &mut Scene, _app: &dyn AppState, _app_ctx: &dyn AppContext) {
		self.vbar.layout(ctx.time, ctx.bounds.size(), self.content_height);
		self.update_content_bounds(ctx.bounds.size());
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, _app: &dyn AppState, _app_ctx: &dyn AppContext) {
		self.vbar.draw(im, ctx, resx, self.content_height);
	}

	fn children(&self) -> &[ChildWidget] {
		slice::from_ref(&self.content)
	}
}

impl dto::ScrollPanel {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let content_height = self.content_height;
		let content = self.content.construct(scene, ctx);
		let content = ChildWidget::new(cvmath::Bounds2i::ZERO, content);
		let vbar = ScrollVBar::new();
		scene.create_widget(|key| {
			ctx.insert(name, key);
			ScrollPanel { key, content, content_height, vbar }
		})
	}
}
