use super::*;

const PROP_STATUS: gui::PropKey = gui::PropKey(1);
const PROP_CHECKBOX_VALUE: gui::PropKey = gui::PropKey(2);
const PROP_TOGGLE_VALUE: gui::PropKey = gui::PropKey(3);
const PROP_SLIDER_VALUE: gui::PropKey = gui::PropKey(4);
const PROP_LOW_VALUE: gui::PropKey = gui::PropKey(5);
const PROP_PULSE_VALUE: gui::PropKey = gui::PropKey(6);
const PROP_RADIO_SELECTED: gui::PropKey = gui::PropKey(7);
const PROP_HIGH_FILL: gui::PropKey = gui::PropKey(8);
const PROP_DISABLED_BUTTON_ENABLED: gui::PropKey = gui::PropKey(9);
const PROP_CHECKBOX_ENABLED: gui::PropKey = gui::PropKey(10);
const PROP_LOW_VALUE_TEXT: gui::PropKey = gui::PropKey(11);
const PROP_PULSE_VALUE_TEXT: gui::PropKey = gui::PropKey(12);
const PROP_SLIDER_VALUE_TEXT: gui::PropKey = gui::PropKey(13);

pub struct State {
	pub window: gui::SlotKey,
	shared: Rc<RefCell<shared_state::State>>,
	checkbox_value: bool,
	checkbox_enabled: bool,
	toggle_value: bool,
	disabled_button_enabled: bool,
	swatch_selected: usize,
	pulse_value: f32,
	repeat_clicks: usize,
	button: Option<gui::SlotKey>,
	repeat: Option<gui::SlotKey>,
	disabled: Option<gui::SlotKey>,
	checkbox: Option<gui::SlotKey>,
	toggle: Option<gui::SlotKey>,
	radio_easy: Option<gui::SlotKey>,
	radio_medium: Option<gui::SlotKey>,
	radio_hard: Option<gui::SlotKey>,
	swatches: [Option<gui::SlotKey>; 6],
}

impl State {
	pub fn new(shared: Rc<RefCell<shared_state::State>>, window: gui::SlotKey) -> State {
		let radio_selected = 1;
		State {
			window,
			shared,
			checkbox_value: true,
			checkbox_enabled: radio_selected == 0,
			toggle_value: false,
			disabled_button_enabled: false,
			swatch_selected: 2,
			pulse_value: 0.45,
			repeat_clicks: 0,
			button: None,
			repeat: None,
			disabled: None,
			checkbox: None,
			toggle: None,
			radio_easy: None,
			radio_medium: None,
			radio_hard: None,
			swatches: [None; 6],
		}
	}

	fn slider_value(&self) -> f32 {
		self.shared.borrow().slider_value
	}

	fn radio_selected(&self) -> usize {
		self.shared.borrow().radio_selected
	}

	fn set_status(&self, status: impl Into<String>) {
		self.shared.borrow_mut().status = status.into();
	}

	pub fn set_pulse_value(&mut self, value: f32) {
		self.pulse_value = value;
	}

	pub fn bind_names(&mut self, ctx: &gui::dto::BuildContext) {
		self.button = Some(ctx.key("button").expect("button"));
		self.repeat = Some(ctx.key("repeat").expect("repeat"));
		self.disabled = Some(ctx.key("disabled").expect("disabled"));
		self.checkbox = Some(ctx.key("checkbox").expect("checkbox"));
		self.toggle = Some(ctx.key("toggle").expect("toggle"));
		self.radio_easy = Some(ctx.key("radio_easy").expect("radio_easy"));
		self.radio_medium = Some(ctx.key("radio_medium").expect("radio_medium"));
		self.radio_hard = Some(ctx.key("radio_hard").expect("radio_hard"));
		for (index, key) in self.swatches.iter_mut().enumerate() {
			let name = format!("swatch_{index}");
			*key = Some(ctx.key(&name).expect(&name));
		}
	}
}

impl gui::AppState for State {
	fn scope<'a>(&'a self, _key: gui::SlotKey, _ctx: &dyn gui::AppContext) -> &'a dyn gui::AppState {
		self
	}

	fn scope_mut<'a>(&'a mut self, _key: gui::SlotKey, _ctx: &mut dyn gui::AppContext) -> &'a mut dyn gui::AppState {
		self
	}

	fn prop(&self, key: gui::PropKey, _ctx: &dyn gui::AppContext, f: &mut dyn FnMut(&dyn std::any::Any)) {
		match key {
			PROP_STATUS => {
				let shared = self.shared.borrow();
				f(&shared.status);
			},
			PROP_CHECKBOX_VALUE => f(&self.checkbox_value),
			PROP_CHECKBOX_ENABLED => f(&self.checkbox_enabled),
			PROP_TOGGLE_VALUE => f(&self.toggle_value),
			PROP_DISABLED_BUTTON_ENABLED => f(&self.disabled_button_enabled),
			PROP_SLIDER_VALUE => {
				let value = self.slider_value();
				f(&value);
			},
			PROP_LOW_VALUE => f(&0.28f32),
			PROP_PULSE_VALUE => f(&self.pulse_value),
			PROP_RADIO_SELECTED => {
				let selected = self.radio_selected();
				f(&selected);
			},
			PROP_HIGH_FILL => {
				let fill = high_fill_color(self.slider_value());
				f(&fill);
			},
			PROP_LOW_VALUE_TEXT => f(&format_percentage(0.28)),
			PROP_PULSE_VALUE_TEXT => f(&format_percentage(self.pulse_value)),
			PROP_SLIDER_VALUE_TEXT => f(&format_percentage(self.slider_value())),
			_ => {},
		}
	}

	fn emit(&mut self, event: &dyn gui::AppEvent, _ctx: &mut dyn gui::AppContext) {
		if let Some(event) = event.downcast_ref::<gui::widgets::ButtonClicked>() {
			if Some(event.key) == self.button {
				self.set_status("Button clicked.");
			}
			else if Some(event.key) == self.repeat {
				self.repeat_clicks += 1;
				self.set_status(format!("Repeat clicked {} time{}.", self.repeat_clicks, if self.repeat_clicks == 1 { "" } else { "s" }));
			}
			else if Some(event.key) == self.disabled {
				self.set_status("Disabled button clicked.");
			}
			else if let Some(selected) = self.swatches.iter().position(|&key| key == Some(event.key)) {
				self.swatch_selected = selected;
				self.set_status(format!("Swatch {} selected.", selected + 1));
			}
			return;
		}

		if let Some(event) = event.downcast_ref::<gui::widgets::CheckboxChanged>() {
			if Some(event.key) == self.checkbox {
				self.checkbox_value = event.checked;
				self.set_status(format!("Checkbox row is now {}.", if event.checked { "on" } else { "off" }));
			}
			else if Some(event.key) == self.toggle {
				self.toggle_value = event.checked;
				if !event.checked && self.radio_selected() == 2 {
					self.shared.borrow_mut().radio_selected = 1;
					self.checkbox_enabled = false;
					self.set_status("Another toggle is now off. Hard mode was disabled, so selection moved back to Medium.");
				}
				else {
					self.set_status(format!("Another toggle is now {}.", if event.checked { "on" } else { "off" }));
				}
			}
			return;
		}

		if let Some(event) = event.downcast_ref::<gui::widgets::SliderChanged>() {
			self.shared.borrow_mut().slider_value = event.value;
			self.set_status(format!("Slider moved to {:.0}%.", (event.value * 100.0) as i32));
			return;
		}

		if let Some(event) = event.downcast_ref::<gui::widgets::RadioButtonChanged>() {
			self.shared.borrow_mut().radio_selected = event.selected;
			self.checkbox_enabled = event.selected == 0;
			let label = if Some(event.key) == self.radio_easy {
				"Easy"
			}
			else if Some(event.key) == self.radio_medium {
				"Medium"
			}
			else if Some(event.key) == self.radio_hard {
				"Hard"
			}
			else {
				"Unknown"
			};
			self.set_status(format!("Radio group selected {} (index {}).", label, event.selected));
		}
	}
}

pub fn load(scene: &mut gui::Scene, ctx: &mut gui::dto::BuildContext) -> gui::SlotKey {
	let dto: gui::dto::Widget = serde_json::from_str(include_str!("main_window.json")).unwrap();
	dto.construct(scene, ctx)
}

fn format_percentage(value: f32) -> String {
	format!("{:.0}%", (value.clamp(0.0, 1.0) * 100.0) as i32)
}
