use super::*;

/// Fixed menu bar item width until widgets gain preferred sizes.
pub const MENU_BAR_ITEM_WIDTH: i32 = 88;

const DEFAULT_ENABLED: bool = true;
const TEXT_INSET_X: i32 = 12;
const TEXT_FONT_SIZE: f32 = 15.0;
const TEXT_LINE_HEIGHT: f32 = TEXT_FONT_SIZE * 1.25;
const OUTLINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(18, 20, 24, 0.86));
const ENABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const DISABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(153, 158, 170));
const HOVER_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(70, 76, 88));

/// Menu bar item that owns a retained popup menu reference.
pub struct MenuBarItem {
	key: SlotKey,
	label: StringProperty,
	enabled: Property<bool>,
	menu: SlotKey,
	hover: bool,
}

impl MenuBarItem {
	fn is_enabled(&self, app: &dyn AppState) -> bool {
		self.enabled.copied_or(app, true)
	}

	fn open_menu(&self, ctx: &EventContext, scene: &mut Scene) {
		scene.open_popup(self.key, self.menu, ctx.bounds);
	}
}

impl Widget for MenuBarItem {
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
				if enabled && scene.popup_owned_by_sibling(self.key) {
					self.open_menu(ctx, scene);
				}
			},
			MouseEventKind::Leave => self.hover = false,
			MouseEventKind::ButtonDown { button: MouseButton::LEFT } if enabled => {
				self.hover = true;
				if scene.popup_owned_by(self.key) {
					scene.close_popup(None);
				}
				else {
					self.open_menu(ctx, scene);
				}
			},
			_ => self.hover = enabled && ctx.target == self.key,
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
			bounds.right() - TEXT_INSET_X,
			bounds.bottom(),
		);
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		if self.label.with(app, |label| {
			im.draw_text_box(ctx, &font, &scribe, &text_bounds, d2::TextAlign::MiddleLeft, label);
		}).is_none() {
			im.draw_text_box(ctx, &font, &scribe, &text_bounds, d2::TextAlign::MiddleLeft, "<menu>");
		}
	}
}

impl dto::MenuBarItem {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let label = From::from(self.label);
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let menu = self.menu.construct(scene, ctx);
		let hover = false;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			MenuBarItem { key, label, enabled, menu, hover }
		})
	}
}
