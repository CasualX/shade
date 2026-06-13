use super::*;

const PROP_FLOATING_TITLE: gui::PropKey = gui::PropKey(1);
const PROP_TOP_HIT_VALUE: gui::PropKey = gui::PropKey(2);

pub struct State {
	pub window: gui::SlotKey,
	shared: Rc<RefCell<shared_state::State>>,
	floating_title: String,
	top_hit_value: bool,
	front: Option<gui::SlotKey>,
	top_hit: Option<gui::SlotKey>,
}

impl State {
	pub fn new(shared: Rc<RefCell<shared_state::State>>, window: gui::SlotKey) -> State {
		State {
			window,
			shared,
			floating_title: "Floating Window\n\x1b[font_size=13.0]\x1b[color=#ACB2BC]Still just another prop.".to_owned(),
			top_hit_value: true,
			front: None,
			top_hit: None,
		}
	}

	fn set_status(&self, status: impl Into<String>) {
		self.shared.borrow_mut().status = status.into();
	}

	pub fn bind_names(&mut self, ctx: &gui::dto::BuildContext) {
		self.front = Some(ctx.key("front").expect("front"));
		self.top_hit = Some(ctx.key("top_hit").expect("top_hit"));
	}
}

impl gui::AppState for State {
	fn scope<'a>(&'a self, _key: gui::SlotKey) -> &'a dyn gui::AppState {
		self
	}

	fn scope_mut<'a>(&'a mut self, _key: gui::SlotKey) -> &'a mut dyn gui::AppState {
		self
	}

	fn prop(&self, key: gui::PropKey, f: &mut dyn FnMut(&dyn std::any::Any)) {
		match key {
			PROP_FLOATING_TITLE => f(&self.floating_title),
			PROP_TOP_HIT_VALUE => f(&self.top_hit_value),
			_ => {},
		}
	}

	fn emit(&mut self, event: &dyn gui::UserEvent) {
		if let Some(event) = event.downcast_ref::<gui::widgets::ButtonClicked>() {
			if Some(event.key) == self.front {
				self.floating_title = "Floating Window\n\x1b[font_size=13.0]\x1b[color=#ACB2BC]Front button clicked.".to_owned();
				self.set_status("Front clicked from the floating window.");
			}
			return;
		}

		if let Some(event) = event.downcast_ref::<gui::widgets::CheckboxChanged>() {
			if Some(event.key) == self.top_hit {
				self.top_hit_value = event.checked;
				self.set_status(format!("Top hit is now {}.", if event.checked { "on" } else { "off" }));
			}
		}
	}
}

pub fn load(scene: &mut gui::Scene, ctx: &mut gui::dto::BuildContext) -> gui::SlotKey {
	let dto: gui::dto::Widget = serde_json::from_str(include_str!("floating_window.json")).unwrap();
	dto.construct(scene, ctx)
}
