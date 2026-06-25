use super::*;

/// Fixed menu bar height until widgets gain preferred sizes.
pub const MENU_BAR_HEIGHT: i32 = 32;

const MENU_BAR_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(43, 46, 55));
const MENU_BAR_EDGE: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(83, 88, 101));

/// Horizontal container for [`MenuBarItem`] widgets.
pub struct MenuBar {
	key: SlotKey,
	children: Vec<ChildWidget>,
}

impl Widget for MenuBar {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, _app: &dyn AppState) -> Option<Cursor> {
		Some(Cursor::Default)
	}

	fn layout(&mut self, ctx: &DrawContext, _resx: &dyn Resources, _scene: &mut Scene, _app: &dyn AppState) {
		let mut left = 0;
		for child in &mut self.children {
			child.bounds = cvmath::Bounds2!(left, 0, left + MENU_BAR_ITEM_WIDTH, ctx.bounds.height());
			left += MENU_BAR_ITEM_WIDTH;
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, _app: &dyn AppState) {
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, bounds, MENU_BAR_FILL, shader);
		let edge = cvmath::Bounds2!(bounds.left(), bounds.bottom() - 1, bounds.right(), bounds.bottom());
		im.fill_rect(ctx, edge, MENU_BAR_EDGE, shader);
	}

	fn children(&self) -> &[ChildWidget] {
		&self.children
	}
}

impl dto::MenuBar {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let children = self.children.into_iter().map(|child| {
			let key = child.construct(scene, ctx);
			ChildWidget::new(cvmath::Bounds2i::ZERO, key)
		}).collect();
		scene.create_widget(|key| {
			ctx.insert(name, key);
			MenuBar { key, children }
		})
	}
}
