use super::*;

pub struct RootPanel {
	panel: Panel,
	dragging: Option<SlotKey>,
	offset: cvmath::Vec2i,
}

impl Widget for RootPanel {
	fn key(&self) -> SlotKey {
		self.panel.key()
	}

	fn cursor(&self, app: &dyn AppState) -> Option<Cursor> {
		self.panel.cursor(app)
	}

	fn event(&mut self, event: &InputEvent, ctx: &EventContext, scene: &mut Scene, _app: &mut dyn AppState) {
		let Some(mouse) = event.mouse() else {
			return;
		};
		let point = mouse.pointer - ctx.bounds.mins;
		match mouse.kind {
			MouseEventKind::ButtonDown { button: MouseButton::LEFT } => {
				let Some(child) = self.panel.hit_test(point) else {
					return;
				};
				if !draggable(&child, scene) {
					return;
				}
				if child.key == ctx.target {
					self.dragging = Some(child.key);
					self.offset = point - child.bounds.mins;
					scene.capture_pointer(child.key);
				}
				self.bring_to_front(child.key);
			},
			MouseEventKind::Move if self.dragging.is_some() => {
				let Some(child) = self.panel.children_mut().iter_mut().find(|child| Some(child.key) == self.dragging) else {
					self.dragging = None;
					return;
				};
				let mins = point - self.offset;
				child.bounds = cvmath::Bounds2i::new(mins, mins + child.bounds.size());
			},
			MouseEventKind::ButtonUp { button: MouseButton::LEFT } => {
				self.dragging = None;
				scene.release_pointer();
			},
			_ => {},
		}
	}

	fn layout(&mut self, ctx: &DrawContext, _resx: &dyn Resources, _scene: &mut Scene, _app: &dyn AppState) {
		// Keep the children within the bounds of the root panel
		let container = cvmath::Bounds2i::vec(ctx.bounds.size());
		for child in self.panel.children_mut() {
			child.bounds = contain_bounds(child.bounds, container);
		}
	}

	fn children(&self) -> &[ChildWidget] {
		self.panel.children()
	}
}

fn draggable(child: &ChildWidget, scene: &Scene) -> bool {
	let Some(widget) = scene.get_widget(child.key) else {
		return false;
	};
	widget.hittable()
}

fn contain_bounds(child: cvmath::Bounds2i, container: cvmath::Bounds2i) -> cvmath::Bounds2i {
	let dx = if child.width() > container.width() {
		child.mins.x - container.mins.x
	}
	else if child.mins.x < container.mins.x {
		child.mins.x - container.mins.x
	}
	else if child.maxs.x > container.maxs.x {
		child.maxs.x - container.maxs.x
	}
	else {
		0
	};
	let dy = if child.height() > container.height() {
		child.mins.y - container.mins.y
	}
	else if child.mins.y < container.mins.y {
		child.mins.y - container.mins.y
	}
	else if child.maxs.y > container.maxs.y {
		child.maxs.y - container.maxs.y
	}
	else {
		0
	};
	child + cvmath::Vec2i(-dx, -dy)
}

impl RootPanel {
	pub fn empty(key: SlotKey) -> RootPanel {
		RootPanel {
			panel: Panel::empty(key),
			dragging: None,
			offset: cvmath::Vec2i::ZERO,
		}
	}
	pub fn attach(&mut self, key: SlotKey, bounds: cvmath::Bounds2i, scene: &mut Scene) {
		self.panel.attach(key, bounds, scene);
	}
	pub fn detach(&mut self, key: SlotKey, scene: &mut Scene) -> bool {
		self.panel.detach(key, scene)
	}
	pub fn bring_to_front(&mut self, key: SlotKey) -> bool {
		let children = self.panel.children_mut();
		let Some(index) = children.iter().position(|child| child.key == key) else {
			return false;
		};
		children[index..].rotate_left(1);
		true
	}
}
