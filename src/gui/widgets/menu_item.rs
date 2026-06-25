use super::*;

/// Fixed menu item height until widgets gain preferred sizes.
pub const MENU_ITEM_HEIGHT: i32 = 32;

const DEFAULT_ENABLED: bool = true;
const TEXT_INSET_X: i32 = 12;
const SUBMENU_INSET_X: i32 = 12;
const SUBMENU_GLYPH_WIDTH: i32 = 12;
const TEXT_FONT_SIZE: f32 = 15.0;
const TEXT_LINE_HEIGHT: f32 = TEXT_FONT_SIZE * 1.25;
const OUTLINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(18, 20, 24, 0.86));
const ENABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const DISABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(153, 158, 170));
const HOVER_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(70, 76, 88));

/// Event emitted when a [`MenuItem`] is clicked.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MenuItemClicked {
	/// Key of the menu item that was clicked.
	pub key: SlotKey,
}

impl UserEvent for MenuItemClicked {}

/// Clickable text row in a [`Menu`].
pub struct MenuItem {
	key: SlotKey,
	label: StringProperty,
	enabled: Property<bool>,
	submenu: Option<SlotKey>,
	hover: bool,
}

impl MenuItem {
	fn is_enabled(&self, app: &dyn AppState) -> bool {
		self.enabled.copied_or(app, true)
	}
}

impl Widget for MenuItem {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, app: &dyn AppState) -> Option<Cursor> {
		self.is_enabled(app).then_some(Cursor::Pointer)
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, app: &mut dyn AppState) {
		let Some(mouse) = event.mouse() else {
			return;
		};
		let enabled = self.is_enabled(app);

		match mouse.kind {
			MouseEventKind::Enter => {
				self.hover = enabled;
				if enabled && let Some(submenu) = self.submenu {
					scene.open_popup(self.key, submenu, ctx.bounds);
				}
				else if let Some(menu) = scene.parent(self.key) {
					scene.close_popup(Some(menu));
				}
			},
			MouseEventKind::Leave => self.hover = false,
			_ => self.hover = enabled && ctx.target == self.key,
		}
		if !enabled {
			return;
		}
		if self.hover && matches!(mouse.kind, MouseEventKind::ButtonUp { button: MouseButton::LEFT }) {
			if let Some(submenu) = self.submenu {
				scene.open_popup(self.key, submenu, ctx.bounds);
				return;
			}
			self.hover = false;
			app.emit(&MenuItemClicked { key: self.key });
			scene.close_popup(None);
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) {
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
		let enabled = self.is_enabled(app);
		if enabled && self.hover {
			let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
			im.fill_rect(ctx, bounds, HOVER_FILL, shader);
		}

		let mut scribe = d2::Scribe {
			font_size: TEXT_FONT_SIZE,
			line_height: TEXT_LINE_HEIGHT,
			color: if enabled { ENABLED_TEXT } else { DISABLED_TEXT },
			outline: OUTLINE_COLOR,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);
		let text_bounds = cvmath::Bounds2!(
			bounds.left() + TEXT_INSET_X,
			bounds.top(),
			bounds.right() - TEXT_INSET_X - if self.submenu.is_some() { SUBMENU_GLYPH_WIDTH + SUBMENU_INSET_X } else { 0 },
			bounds.bottom(),
		);
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		if self.label.with(app, |label| {
			im.draw_text_box(ctx, &font, &scribe, &text_bounds, d2::TextAlign::MiddleLeft, label);
		}).is_none() {
			im.draw_text_box(ctx, &font, &scribe, &text_bounds, d2::TextAlign::MiddleLeft, "<menu item>");
		}
		if self.submenu.is_some() {
			let glyph_bounds = cvmath::Bounds2!(
				bounds.right() - SUBMENU_INSET_X - SUBMENU_GLYPH_WIDTH,
				bounds.top(),
				bounds.right() - SUBMENU_INSET_X,
				bounds.bottom(),
			);
			im.draw_text_box(ctx, &font, &scribe, &glyph_bounds, d2::TextAlign::MiddleCenter, ">");
		}
	}
}

impl dto::MenuItem {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let label = From::from(self.label);
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let submenu = self.submenu.map(|submenu| submenu.construct(scene, ctx));
		let hover = false;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			MenuItem { key, label, enabled, submenu, hover }
		})
	}
}
