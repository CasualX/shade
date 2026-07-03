use super::*;

pub struct State {
	pub window: gui::SlotKey,
	shared: Rc<RefCell<shared_state::State>>,
}

impl State {
	pub fn new(shared: Rc<RefCell<shared_state::State>>, window: gui::SlotKey) -> State {
		State {
			window,
			shared,
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

	fn prop(&self, _key: gui::PropKey, _ctx: &dyn gui::AppContext, _f: &mut dyn FnMut(&dyn std::any::Any)) {}

	fn emit(&mut self, event: &dyn gui::AppEvent, _ctx: &mut dyn gui::AppContext) {
		if let Some(_event) = event.downcast_ref::<gui::widgets::ButtonClicked>() {
			self.shared.borrow_mut().status = "Drawing window button clicked.".to_owned();
		}
	}
}

pub fn load(scene: &mut gui::Scene, ctx: &mut gui::dto::BuildContext) -> gui::SlotKey {
	let dto: gui::dto::Widget = serde_json::from_str(include_str!("drawing_window.json")).unwrap();
	dto.construct(scene, ctx)
}
