use super::*;

const DEFAULT_ENABLED: bool = true;
const BOX_SIZE: i32 = 24;
const CHECK_INSET: i32 = 5;
const LABEL_OFFSET_X: i32 = 34;
const TEXT_FONT_SIZE: f32 = 15.0;
const TEXT_LINE_HEIGHT: f32 = TEXT_FONT_SIZE * 1.25;
const OUTLINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(18, 20, 24));
const DISABLED_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(74, 79, 92));
const HOVER_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(128, 159, 198));
const DEFAULT_RING: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(93, 99, 113));
const ENABLED_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(92, 150, 218));
const DISABLED_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(90, 103, 121));
const ENABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const DISABLED_TEXT: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(153, 158, 170));
const EDGE_THICKNESS: f32 = 1.0;

/// Checkbox widget with a text label.
pub struct Checkbox {
	key: SlotKey,
	label: StringProperty,
	checked: Property<bool>,
	enabled: Property<bool>,
	hover: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Event emitted when a [`Checkbox`] toggles.
pub struct CheckboxChanged {
	/// Key of the checkbox that changed.
	pub key: SlotKey,
	/// New checked state.
	pub checked: bool,
}

impl UserEvent for CheckboxChanged {}

impl Checkbox {
	fn is_enabled(&self, app: &dyn AppState) -> bool {
		self.enabled.copied_or(app, true)
	}
}

impl Widget for Checkbox {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, app: &dyn AppState) -> Option<Cursor> {
		if self.is_enabled(app) {
			Some(Cursor::Pointer)
		}
		else {
			None
		}
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, _scene: &mut Scene, app: &mut dyn AppState) {
		let Some(mouse) = event.mouse() else {
			return;
		};

		if !self.is_enabled(app) {
			self.hover = false;
			return;
		}

		match mouse.kind {
			MouseEventKind::Enter => self.hover = true,
			MouseEventKind::Leave => self.hover = false,
			_ => self.hover = ctx.target == self.key,
		}
		if self.hover && matches!(mouse.kind, MouseEventKind::ButtonUp { button: MouseButton::LEFT }) {
			let checked = !self.checked.copied_or(app, false);
			let changed = CheckboxChanged {
				key: self.key,
				checked,
			};
			app.emit(&changed);
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) {
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let box_top = rc.top() + (rc.height() - BOX_SIZE) / 2;
		let box_rc = cvmath::Bounds2!(rc.left(), box_top, rc.left() + BOX_SIZE, box_top + BOX_SIZE);
		let enabled = self.is_enabled(app);
		let color = if !enabled { DISABLED_RING }
		else if self.hover { HOVER_RING }
		else { DEFAULT_RING };
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_edge_rect(ctx, box_rc, color, EDGE_THICKNESS, shader);
		if self.checked.copied_or(app, false) {
			let fill = if enabled { ENABLED_FILL } else { DISABLED_FILL };
			im.fill_rect(ctx, box_rc.inset(CHECK_INSET), fill, shader);
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
		if self.label.with(app, |text| {
			im.draw_text_box(ctx, &font, &scribe, &rc, d2::TextAlign::MiddleLeft, text);
		}).is_none() {
			im.draw_text_box(ctx, &font, &scribe, &rc, d2::TextAlign::MiddleLeft, "<checkbox>");
		}
	}
}

impl dto::Checkbox {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let label = From::from(self.label);
		let checked = From::from(self.checked);
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let hover = false;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Checkbox { key, label, checked, enabled, hover }
		})
	}
}
