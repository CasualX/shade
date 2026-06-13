pub(super) struct State {
	pub status: String,
	pub slider_value: f32,
	pub radio_selected: usize,
}

impl State {
	pub(super) fn new() -> State {
		State {
			status: "Try the controls and watch this label update from semantic events.".to_owned(),
			slider_value: 0.62,
			radio_selected: 1,
		}
	}
}
