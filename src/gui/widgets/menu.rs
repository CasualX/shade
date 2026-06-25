use super::*;

/// Fixed width used by menus until widgets gain preferred sizes.
pub const MENU_WIDTH: i32 = 240;

const MENU_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(37, 40, 48));
const MENU_EDGE: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(83, 88, 101));
const EDGE_THICKNESS: f32 = 1.0;

/// Vertical menu that positions menu items and separators.
pub struct Menu {
	key: SlotKey,
	children: Vec<ChildWidget>,
	height: i32,
}

impl Widget for Menu {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, _app: &dyn AppState) -> Option<Cursor> {
		Some(Cursor::Default)
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, _app: &mut dyn AppState) {
		let Some(mouse) = event.mouse() else {
			return;
		};
		if ctx.target == self.key && matches!(mouse.kind, MouseEventKind::Enter | MouseEventKind::Move) {
			scene.close_popup(Some(self.key));
		}
	}

	fn layout(&mut self, _ctx: &DrawContext, _resx: &dyn Resources, scene: &mut Scene, _app: &dyn AppState) {
		let mut top = 0;
		for child in &mut self.children {
			let height = if scene.get_widget(child.key).is_some_and(|widget| widget.downcast_ref::<Separator>().is_some()) {
				SEPARATOR_HEIGHT
			}
			else {
				MENU_ITEM_HEIGHT
			};
			child.bounds = cvmath::Bounds2!(0, top, MENU_WIDTH, top + height);
			top += height;
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, _app: &dyn AppState) {
		let bounds = cvmath::Bounds2!(0, 0, ctx.bounds.width(), ctx.bounds.height());
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, bounds, MENU_FILL, shader);
		im.fill_edge_rect(ctx, bounds, MENU_EDGE, EDGE_THICKNESS, shader);
	}

	fn children(&self) -> &[ChildWidget] {
		&self.children
	}
}

impl dto::Menu {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let height = self.children.iter().map(|child| {
			if matches!(child, dto::Widget::Separator(_)) { SEPARATOR_HEIGHT } else { MENU_ITEM_HEIGHT }
		}).sum();
		let children = self.children.into_iter().map(|child| {
			let key = child.construct(scene, ctx);
			ChildWidget::new(cvmath::Bounds2i::ZERO, key)
		}).collect();
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Menu { key, children, height }
		})
	}
}

impl Menu {
	pub(crate) fn height(&self) -> i32 {
		self.height
	}
}
