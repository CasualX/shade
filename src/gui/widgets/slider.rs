use super::*;

const DEFAULT_ENABLED: bool = true;
const TRACK_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(26, 28, 34));
const DISABLED_TRACK_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(32, 34, 40));
const FILL_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(92, 150, 218));
const DISABLED_FILL_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(90, 103, 121));
const DISABLED_KNOB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(123, 130, 143));
const DRAGGING_KNOB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(230, 232, 236));
const HOVER_KNOB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(202, 221, 242));
const DEFAULT_KNOB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(177, 203, 232));

#[derive(Copy, Clone, Debug, PartialEq)]
/// Event emitted when a [`Slider`] value changes.
pub struct SliderChanged {
	/// Key of the slider that changed.
	pub key: SlotKey,
	/// New normalized value in the range `0.0..=1.0`.
	pub value: f32,
}

impl UserEvent for SliderChanged {}

/// Horizontal slider widget.
pub struct Slider {
	key: SlotKey,
	value: Property<f32>,
	enabled: Property<bool>,
	dragging: bool,
	hover: bool,
}

const TRACK_HEIGHT: i32 = 6;
const KNOB_WIDTH: i32 = 10;
const KNOB_HEIGHT: i32 = 20;

impl Slider {
	fn track_rect(&self, bounds: cvmath::Bounds2i) -> cvmath::Bounds2i {
		let track_left = bounds.left();
		let track_top = bounds.top() + (bounds.height() - TRACK_HEIGHT) / 2;
		let track_width = bounds.width().max(1);
		cvmath::Bounds2!(track_left, track_top, track_left + track_width, track_top + TRACK_HEIGHT)
	}

	fn knob_left(&self, track: cvmath::Bounds2i, value: f32) -> i32 {
		let value = value.clamp(0.0, 1.0);
		let travel = track.width() - KNOB_WIDTH;
		if travel <= 0 {
			return track.left();
		}
		track.left() + (value * travel as f32).round() as i32
	}

	fn value_from_point(&self, point: cvmath::Vec2i, bounds: cvmath::Bounds2i) -> f32 {
		let track = self.track_rect(cvmath::Bounds2i::vec(bounds.size()));
		let travel = track.width() - KNOB_WIDTH;
		if travel <= 0 {
			return 0.0;
		}
		let knob_left = (point.x - bounds.mins.x - KNOB_WIDTH / 2).clamp(track.left(), track.right() - KNOB_WIDTH);
		(knob_left - track.left()) as f32 / travel as f32
	}

	fn is_enabled(&self, app: &dyn AppState) -> bool {
		self.enabled.copied_or(app, true)
	}
}

impl Widget for Slider {
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

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, app: &mut dyn AppState) {
		let Some(mouse) = event.mouse() else {
			return;
		};

		if !self.is_enabled(app) {
			self.dragging = false;
			self.hover = false;
			scene.release_pointer();
			return;
		}

		match mouse.kind {
			MouseEventKind::Enter => self.hover = true,
			MouseEventKind::Leave => self.hover = false,
			_ => self.hover = ctx.target == self.key,
		}
		if self.hover {
			match mouse.kind {
				MouseEventKind::ButtonDown { button: MouseButton::LEFT } => {
					self.dragging = true;
					scene.capture_pointer(self.key);
					let changed = SliderChanged {
						key: self.key,
						value: self.value_from_point(mouse.pointer, ctx.bounds),
					};
					app.emit(&changed);
				},
				MouseEventKind::ButtonUp { button: MouseButton::LEFT } => {
					let changed = SliderChanged {
						key: self.key,
						value: self.value_from_point(mouse.pointer, ctx.bounds),
					};
					app.emit(&changed);
					self.dragging = false;
					scene.release_pointer();
				},
				MouseEventKind::Move if self.dragging => {
					let changed = SliderChanged {
						key: self.key,
						value: self.value_from_point(mouse.pointer, ctx.bounds),
					};
					app.emit(&changed);
				},
				_ => {},
			}
		}
		else if matches!(mouse.kind, MouseEventKind::ButtonUp { button: MouseButton::LEFT }) {
			self.dragging = false;
			scene.release_pointer();
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) {
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let enabled = self.is_enabled(app);
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();

		let track = self.track_rect(rc);
		let value = self.value.copied_or(app, 0.0).clamp(0.0, 1.0);
		im.fill_rect(ctx, track, if enabled { TRACK_COLOR } else { DISABLED_TRACK_COLOR }, shader);
		let fill_right = track.left() + (value * track.width() as f32) as i32;
		let fill_rc = cvmath::Bounds2!(track.left(), track.top(), fill_right, track.bottom());
		im.fill_rect(ctx, fill_rc, if enabled { FILL_COLOR } else { DISABLED_FILL_COLOR }, shader);

		let knob_left = self.knob_left(track, value);
		let knob_top = rc.top() + (rc.height() - KNOB_HEIGHT) / 2;
		let knob_color = if !enabled { DISABLED_KNOB_COLOR }
		else if self.dragging { DRAGGING_KNOB_COLOR }
		else if self.hover { HOVER_KNOB_COLOR }
		else { DEFAULT_KNOB_COLOR };
		let knob_rc = cvmath::Bounds2!(knob_left, knob_top, knob_left + KNOB_WIDTH, knob_top + KNOB_HEIGHT);
		im.fill_rect(ctx, knob_rc, knob_color, shader);
	}
}

impl dto::Slider {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let value = From::from(self.value);
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let dragging = false;
		let hover = false;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Slider { key, value, enabled, dragging, hover }
		})
	}
}
