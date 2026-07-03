use super::*;

/// Event emitted when a [`Button`] is clicked.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ButtonClicked {
	/// Key of the button that was clicked.
	pub key: SlotKey,
}

impl AppEvent for ButtonClicked {}

const DEFAULT_ENABLED: bool = true;
const DEFAULT_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(54, 58, 68));
const HOVER_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(70, 76, 88));
const PRESSED_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(64, 91, 124));
const DISABLED_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(44, 47, 56));
const DEFAULT_EDGE: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(88, 94, 108));
const ACTIVE_EDGE: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(128, 159, 198));
const DISABLED_EDGE: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(74, 79, 92));
const EDGE_THICKNESS: f32 = 1.0;

/// Visual state of a button.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ButtonState {
	Normal,
	Hover,
	Pressed,
}

/// Clickable button widget.
pub struct Button {
	key: SlotKey,
	content: ChildWidget,
	enabled: Property<bool>,
	state: ButtonState,
}

impl Button {
	fn is_enabled(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> bool {
		self.enabled.copied_or(app, app_ctx, true)
	}
}

impl Widget for Button {
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

	fn layout(&mut self, ctx: &DrawContext, _resx: &dyn Resources, _scene: &mut Scene, _app: &dyn AppState, _app_ctx: &dyn AppContext) {
		self.content.bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, _scene: &mut Scene, app: &mut dyn AppState, app_ctx: &mut dyn AppContext) {
		let Some(mouse) = event.mouse() else {
			return;
		};

		let hover = matches!(ctx.target, target if target == self.key || target == self.content.key);
		let was_pressed = self.state == ButtonState::Pressed;
		if !self.is_enabled(app, app_ctx) {
			self.state = ButtonState::Normal;
			return;
		}

		match mouse.kind {
			MouseEventKind::Enter => {
				self.state = if was_pressed { ButtonState::Pressed } else { ButtonState::Hover };
			},
			MouseEventKind::Leave => {
				self.state = ButtonState::Normal;
			},
			MouseEventKind::ButtonDown { button: MouseButton::LEFT } if hover => {
				self.state = ButtonState::Pressed;
			},
			MouseEventKind::ButtonUp { button: MouseButton::LEFT } => {
				if was_pressed && hover {
					let clicked = ButtonClicked { key: self.key };
					app.emit(&clicked, app_ctx);
				}
				self.state = if hover { ButtonState::Hover } else { ButtonState::Normal };
			},
			MouseEventKind::Move => {
				self.state = if was_pressed && hover { ButtonState::Pressed}
				else if hover { ButtonState::Hover }
				else { ButtonState::Normal };
			},
			_ if !hover => {
				self.state = ButtonState::Normal;
			},
			_ if !was_pressed => {
				self.state = ButtonState::Hover;
			},
			_ => {},
		}
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState, app_ctx: &dyn AppContext) {
		let enabled = self.is_enabled(app, app_ctx);
		let fill = if !enabled { DISABLED_FILL }
		else if self.state == ButtonState::Pressed { PRESSED_FILL }
		else if self.state == ButtonState::Hover { HOVER_FILL }
		else { DEFAULT_FILL };
		let edge = if !enabled { DISABLED_EDGE }
		else if self.state == ButtonState::Hover || self.state == ButtonState::Pressed { ACTIVE_EDGE }
		else { DEFAULT_EDGE };
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, bounds, fill, shader);
		im.fill_edge_rect(ctx, bounds, edge, EDGE_THICKNESS, shader);
	}

	fn children(&self) -> &[ChildWidget] {
		slice::from_ref(&self.content)
	}
}

impl dto::Button {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let content = self.content.construct(scene, ctx);
		let content = ChildWidget::new(cvmath::Bounds2i::ZERO, content); // Bounds are computed during layout
		let enabled = self.enabled.map(From::from).unwrap_or(Property::Value(DEFAULT_ENABLED));
		let state = ButtonState::Normal;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Button { key, content, enabled, state }
		})
	}
}
