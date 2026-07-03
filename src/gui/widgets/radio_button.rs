use super::*;

const DEFAULT_ENABLED: bool = true;
const OUTER_SIZE: i32 = 22;
const FILL_INSET: i32 = 6;
const LABEL_OFFSET_X: i32 = 32;
const TEXT_FONT_SIZE: f32 = 15.0;
const TEXT_LINE_HEIGHT: f32 = TEXT_FONT_SIZE * 1.25;
const OUTLINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(18, 20, 24, 0.86));
const DISABLED_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(74, 79, 92));
const CHECKED_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(92, 150, 218));
const HOVER_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(128, 159, 198));
const DEFAULT_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(93, 99, 113));
const DISABLED_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(90, 103, 121));
const ENABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const DISABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(153, 158, 170));
const EDGE_THICKNESS: f32 = 1.0;

/// Radio button widget with a text label.
pub struct RadioButton {
	key: SlotKey,
	label: StringProperty,
	property: Property<usize>,
	index: usize,
	enabled: Property<bool>,
	hover: bool,
}

/// Event emitted when a [`RadioButton`] selects its index.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RadioButtonChanged {
	/// Key of the radio button that was clicked.
	pub key: SlotKey,
	/// Property key of the selected index.
	pub property: Property<usize>,
	/// New selected index for the group.
	pub selected: usize,
}

impl AppEvent for RadioButtonChanged {}

impl RadioButton {
	fn checked(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> bool {
		self.property.copied(app, app_ctx) == Some(self.index)
	}

	fn is_enabled(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> bool {
		self.enabled.copied_or(app, app_ctx, true)
	}
}

impl Widget for RadioButton {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> Option<Cursor> {
		if self.is_enabled(app, app_ctx) {
			Some(Cursor::Pointer)
		}
		else {
			None
		}
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, _scene: &mut Scene, app: &mut dyn AppState, app_ctx: &mut dyn AppContext) {
		let Some(mouse) = event.mouse() else {
			return;
		};

		if !self.is_enabled(app, app_ctx) {
			self.hover = false;
			return;
		}

		match mouse.kind {
			MouseEventKind::Enter => self.hover = true,
			MouseEventKind::Leave => self.hover = false,
			_ => self.hover = ctx.target == self.key,
		}
		if self.hover && matches!(mouse.kind, MouseEventKind::ButtonUp { button: MouseButton::LEFT }) && !self.checked(app, app_ctx) {
			let changed = RadioButtonChanged {
				key: self.key,
				property: self.property,
				selected: self.index,
			};
			app.emit(&changed, app_ctx);
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState, app_ctx: &dyn AppContext) {
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let outer_top = rc.top() + (rc.height() - OUTER_SIZE) / 2;
		let outer = cvmath::Bounds2!(rc.left(), outer_top, rc.left() + OUTER_SIZE, outer_top + OUTER_SIZE);
		let enabled = self.is_enabled(app, app_ctx);
		let checked = self.checked(app, app_ctx);
		let ring = if !enabled { DISABLED_RING }
		else if checked { CHECKED_RING }
		else if self.hover { HOVER_RING }
		else { DEFAULT_RING };
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_edge_rect(ctx, outer, ring, EDGE_THICKNESS, shader);
		if checked {
			let fill = if enabled { CHECKED_RING } else { DISABLED_FILL };
			im.fill_rect(ctx, outer.inset(FILL_INSET), fill, shader);
		}

		let mut scribe = d2::Scribe {
			font_size: TEXT_FONT_SIZE,
			line_height: TEXT_LINE_HEIGHT,
			color: if enabled { ENABLED_TEXT } else { DISABLED_TEXT },
			outline: OUTLINE_COLOR,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);
		let rc = cvmath::Bounds2!(rc.left() + LABEL_OFFSET_X, rc.top(), rc.right(), rc.bottom());
		let font = resx.get_font(SystemResources::FONT_KEY).unwrap();
		if self.label.with(app, app_ctx, |text| {
			im.draw_text_box(ctx, &font, &scribe, &rc, d2::TextAlign::MiddleLeft, text);
		}).is_none() {
			im.draw_text_box(ctx, &font, &scribe, &rc, d2::TextAlign::MiddleLeft, "<radio>");
		}
	}
}

impl dto::RadioButton {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let label = From::from(self.label);
		let selected_index = From::from(self.selected);
		let index = self.index;
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let hover = false;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			RadioButton { key, label, property: selected_index, index, enabled, hover }
		})
	}
}
