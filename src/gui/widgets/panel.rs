use super::*;

/// Container widget that forwards layout, events, and drawing to its children.
pub struct Panel {
	key: SlotKey,
	children: Vec<ChildWidget>,
}

impl Widget for Panel {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn cursor(&self, _app: &dyn AppState, _app_ctx: &dyn AppContext) -> Option<Cursor> {
		Some(Cursor::Default)
	}

	fn children(&self) -> &[ChildWidget] {
		&self.children
	}
}

impl Panel {
	pub(crate) fn empty(key: SlotKey) -> Panel {
		Panel {
			key,
			children: Vec::new(),
		}
	}

	pub(crate) fn hit_test(&self, point: cvmath::Point2i) -> Option<&ChildWidget> {
		self.children.iter().rev().find(|child| child.bounds.contains(point))
	}

	pub(crate) fn children_mut(&mut self) -> &mut Vec<ChildWidget> {
		&mut self.children
	}

	pub fn attach(&mut self, key: SlotKey, bounds: cvmath::Bounds2i, scene: &mut Scene) {
		if let Some(child) = self.children.iter_mut().find(|child| child.key == key) {
			child.bounds = bounds;
		}
		else {
			self.children.push(ChildWidget::new(bounds, key));
			scene.set_parent(key, self.key);
		}
	}

	pub fn detach(&mut self, key: SlotKey, scene: &mut Scene) -> bool {
		let Some(index) = self.children.iter().position(|child| child.key == key) else {
			return false;
		};
		self.children.remove(index);
		scene.clear_parent(key);
		true
	}
}

impl dto::Panel {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let children = self.children.into_iter().map(|child| {
			let key = child.widget.construct(scene, ctx);
			ChildWidget::new(child.bounds, key)
		}).collect();
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Panel { key, children }
		})
	}
}
