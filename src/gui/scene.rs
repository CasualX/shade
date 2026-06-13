use super::*;

/// Retained GUI scene containing all widgets and root ordering.
pub struct Scene {
	slots: SlotMap<Box<dyn Widget>>,
	parents: Vec<Option<SlotKey>>,
	size: cvmath::Vec2i,
	content: Option<SlotKey>,
	// Future idea: focused widget / keyboard routing.
	// focused: Option<SlotKey>,
	pointed: Option<SlotKey>,
	captured: Option<SlotKey>,
}

impl Scene {
	/// Creates a new instance with given size.
	pub fn new(size: cvmath::Vec2i) -> Scene {
		let mut scene = Scene {
			slots: SlotMap::new(),
			parents: Vec::new(),
			size,
			content: None,
			// focused: None,
			pointed: None,
			captured: None,
		};
		scene.content = Some(scene.create_widget(widgets::RootPanel::empty));
		scene
	}

	/// Allocates a key, constructs a widget, and registers parent links for its current children.
	///
	/// The factory receives the allocated key so widgets can store stable self-identity at
	/// creation time. Any children returned by [`Widget::children`] immediately after construction
	/// are recorded as belonging to the new widget.
	pub fn create_widget<T: Widget>(&mut self, f: impl FnOnce(SlotKey) -> T) -> SlotKey {
		let key = self.slots.alloc();
		let widget = Box::new(f(key));
		assert_eq!(widget.key(), key, "Widget key does not match allocated slot");
		for child in widget.children() {
			self.set_parent(child.key, key);
		}
		self.put_widget(key, widget);
		key
	}

	/// Recursively destroys a widget and all of its descendants.
	///
	/// This clears the parent metadata for the whole subtree and frees every occupied slot.
	pub fn destroy_widget(&mut self, key: SlotKey) {
		// if self.focused == Some(key) {
		// 	self.focused = None;
		// }
		if self.pointed == Some(key) {
			self.pointed = None;
		}
		if self.captured == Some(key) {
			self.captured = None;
		}
		self.clear_parent(key);
		let widget = self.slots.remove(key);
		for child in widget.children() {
			self.destroy_widget(child.key);
		}
	}

	/// Returns the widget with the given key, if any.
	pub fn get_widget(&self, key: SlotKey) -> Option<&dyn Widget> {
		self.slots.get(key).map(|widget| widget.as_ref())
	}

	/// Returns the widget mutably with the given key, if any.
	pub fn get_widget_mut(&mut self, key: SlotKey) -> Option<&mut dyn Widget> {
		self.slots.get_mut(key).map(|widget| widget.as_mut())
	}

	/// Temporarily takes ownership of a widget. Low-level API, use with care.
	///
	/// The slot remains reserved and must be filled again with [`put_widget`](Scene::put_widget).
	///
	/// Instead, consider using [`with_widget`](Scene::with_widget) to handle the widget safely.
	pub fn take_widget(&mut self, key: SlotKey) -> Box<dyn Widget> {
		self.slots.take(key)
	}

	/// Puts a temporarily taken widget back into its slot. Low-level API, use with care.
	///
	/// Instead, consider using [`with_widget`](Scene::with_widget) to handle the widget safely.
	pub fn put_widget(&mut self, key: SlotKey, widget: Box<dyn Widget>) {
		self.slots.put(key, widget);
	}

	/// Temporarily takes ownership of a widget, runs `f` given the widget and scene, and then puts it back.
	pub fn with_widget<R>(&mut self, key: SlotKey, f: impl FnOnce(&mut dyn Widget, &mut Scene) -> R) -> R {
		let value = self.take_widget(key);
		struct Guard<'a> {
			key: SlotKey,
			scene: &'a mut Scene,
			value: Option<Box<dyn Widget>>,
		}
		impl<'a> Drop for Guard<'a> {
			fn drop(&mut self) {
				if let Some(value) = self.value.take() {
					self.scene.put_widget(self.key, value);
				}
			}
		}
		let mut guard = Guard { key, scene: self, value: Some(value) };
		f(&mut **guard.value.as_mut().unwrap(), guard.scene)
	}

	/// Resizes the scene to the given size.
	pub fn resize(&mut self, size: cvmath::Vec2i) {
		self.size = size;
	}

	/// Temporarily takes ownership of the root panel, runs `f` given the root panel and scene, and then puts it back.
	fn with<R>(&mut self, f: impl FnOnce(&mut widgets::RootPanel, &mut Scene) -> R) -> R {
		let content = self.content.expect("scene root panel is not initialized");
		self.with_widget(content, |widget, scene| {
			let root = widget.downcast_mut::<widgets::RootPanel>().unwrap();
			f(root, scene)
		})
	}

	/// Shows the widget on the root panel with given bounds.
	///
	/// Panics if the widget does not exist or is attached to another parent.
	///
	/// The widget is attached to the root panel and becomes visible.
	/// If it was already attached its bounds are updated to the new value.
	pub fn show(&mut self, key: SlotKey, bounds: cvmath::Bounds2i) {
		assert!(self.get_widget(key).is_some(), "Cannot show unknown widget {key:?}");
		let parent = self.parent(key);
		if parent.is_some() && parent != self.content {
			panic!("Cannot show widget {key:?} with parent {parent:?} other than the root panel");
		}
		self.with(|root, scene| {
			root.attach(key, bounds, scene);
		});
	}

	/// Hides the widget from the root panel.
	///
	/// Panics if the widget does not exists or is not currently attached to the root panel.
	///
	/// The widget is detached from the root panel and not destroyed.
	pub fn hide(&mut self, key: SlotKey) {
		assert!(self.get_widget(key).is_some(), "Cannot hide unknown widget {key:?}");
		let parent = self.parent(key);
		if parent != self.content {
			panic!("Cannot hide widget {key:?} with parent {parent:?} other than the root panel");
		}
		self.with(|root, scene| {
			root.detach(key, scene);
		});
	}

	/// Returns the bounds clipped in the scene of the given widget.
	///
	/// Query can fail if the widget is not found or attached or if the widget is clipped out of its parent bounds.
	pub fn get_client_rect(&self, widget: SlotKey) -> Option<cvmath::Bounds2i> {
		let root_bounds = cvmath::Bounds2i::vec(self.size);
		get_client_rect(widget, &root_bounds, self)
	}

	/// Returns the widget's key at the given point, if any.
	pub fn hit_test(&self, point: cvmath::Vec2i) -> Option<SlotKey> {
		if !cvmath::Bounds2i::vec(self.size).contains(point) {
			return None;
		}
		hit_test(self.content.expect("scene root panel is not initialized"), point, self)
	}

	/// Returns the cursor to show, if any.
	pub fn get_cursor(&self, app: &dyn AppState) -> Option<Cursor> {
		let pointed = self.pointed?;
		let (cursor, _) = get_cursor_scoped(pointed, app, self)?;
		cursor
	}

	/// Captures mouse routing to `target` until it is released again.
	pub fn capture_pointer(&mut self, target: SlotKey) {
		self.captured = Some(target);
	}

	/// Releases any current mouse capture.
	pub fn release_pointer(&mut self) {
		self.captured = None;
	}

	/// Handles mouse input, including hover tracking and pointer capture routing.
	pub fn mouse_event(&mut self, event: &MouseEvent, time: time::Instant, app: &mut dyn AppState) {
		let input = InputEvent::Mouse(event.clone());
		let hovered = self.hit_test(event.pointer);

		if let Some(captured) = self.captured {
			self.update_pointed(Some(captured), event.pointer, time, app);
			self.event(captured, time, &input, app);
			if self.captured.is_none() {
				self.update_pointed(hovered, event.pointer, time, app);
			}
			return;
		}

		self.update_pointed(hovered, event.pointer, time, app);
		if let Some(target) = hovered {
			self.event(target, time, &input, app);
		}
	}

	fn update_pointed(&mut self, new: Option<SlotKey>, pointer: cvmath::Vec2i, time: time::Instant, app: &mut dyn AppState) {
		let old = self.pointed;
		if old == new {
			return;
		}
		self.pointed = new;
		if let Some(old) = old {
			self.event(old, time, &InputEvent::Mouse(MouseEvent { kind: MouseEventKind::Leave, pointer }), app);
		}
		if let Some(new) = new {
			self.event(new, time, &InputEvent::Mouse(MouseEvent { kind: MouseEventKind::Enter, pointer }), app);
		}
	}

	/// Delivers an input event to the target widget.
	pub fn event(&mut self, target: SlotKey, time: time::Instant, event: &InputEvent, app: &mut dyn AppState) {
		let mut route_buf = [const { mem::MaybeUninit::uninit() }; 120];
		let route = route_to(target, self, &mut route_buf);
		let mut ctx = RouteEventContext { time, target, bounds: cvmath::Bounds2i::vec(self.size), event };
		route_event(&mut ctx, route, app, self);
	}

	/// Layouts the scene and all widgets recursively.
	pub fn layout(&mut self, time: time::Instant, resx: &dyn Resources, app: &dyn AppState) {
		let ctx = DrawContext {
			viewport: cvmath::Bounds2i::vec(self.size),
			clip: cvmath::Bounds2i::vec(self.size),
			time,
			bounds: cvmath::Bounds2i::vec(self.size),
		};
		layout_tree(self.content.expect("scene root panel is not initialized"), &ctx, resx, app, self);
	}

	/// Draws the scene.
	pub fn draw<'a>(&mut self, g: &mut Graphics, im: &mut im::DrawPool<'a>, time: time::Instant, resx: &'a dyn Resources, app: &dyn AppState) {
		let ctx = DrawContext {
			viewport: cvmath::Bounds2i::vec(self.size),
			clip: cvmath::Bounds2i::vec(self.size),
			time,
			bounds: cvmath::Bounds2i::vec(self.size),
		};
		draw_tree(g, im, self.content.expect("scene root panel is not initialized"), &ctx, resx, app, self);
	}

	/// Brings the top level widget with the given key to the front of the root panel.
	pub fn bring_to_front(&mut self, key: SlotKey) -> bool {
		self.with(|root, _scene| root.bring_to_front(key))
	}

	/// Returns the parent of the given widget, if any.
	pub fn parent(&self, key: SlotKey) -> Option<SlotKey> {
		self.parents.get(key.index()).copied().flatten()
	}

	pub(super) fn set_parent(&mut self, child: SlotKey, parent: SlotKey) {
		let index = child.index();
		if self.parents.len() <= index {
			self.parents.resize(index + 1, None);
		}
		let _old = mem::replace(&mut self.parents[index], Some(parent));
		#[cfg(debug_assertions)]
		if _old.is_some() {
			panic!("Widget {child:?} already has a parent");
		}
	}

	pub(super) fn clear_parent(&mut self, child: SlotKey) {
		if let Some(parent) = self.parents.get_mut(child.index()) {
			*parent = None;
		}
	}
}

impl dto::Scene {
	/// Builds a retained GUI scene from this declarative scene.
	pub fn build(self) -> (Scene, dto::BuildContext) {
		let mut ctx = dto::BuildContext::new();
		let scene = self.construct(&mut ctx);
		(scene, ctx)
	}

	/// Builds a retained GUI scene and records named widget keys into `ctx`.
	pub fn construct(self, ctx: &mut dto::BuildContext) -> Scene {
		let mut scene = Scene::new(self.size);
		for root in self.roots {
			let key = root.widget.construct(&mut scene, ctx);
			scene.show(key, root.bounds);
		}
		scene
	}
}

fn get_client_rect(key: SlotKey, root_bounds: &cvmath::Bounds2i, scene: &Scene) -> Option<cvmath::Bounds2i> {
	let Some(parent) = scene.parent(key) else {
		return Some(*root_bounds);
	};
	let parent_bounds = get_client_rect(parent, root_bounds, scene)?;
	let parent = scene.get_widget(parent)?;
	let local_bounds = parent.children().iter().find(|&child| child.key == key)?.bounds;
	let bounds = local_bounds + parent_bounds.mins;
	bounds.intersect(parent_bounds)
}

fn hit_test(key: SlotKey, point: cvmath::Vec2i, scene: &Scene) -> Option<SlotKey> {
	let widget = scene.get_widget(key)?;
	if !widget.hittable() {
		return None;
	}

	for child in widget.children().iter().rev() {
		if child.bounds.contains(point) {
			let local_point = point - child.bounds.mins;
			if let Some(hit) = hit_test(child.key, local_point, scene) {
				return Some(hit);
			}
		}
	}

	Some(key)
}

struct RouteEventContext<'a> {
	target: SlotKey,
	event: &'a InputEvent,
	time: time::Instant,
	bounds: cvmath::Bounds2i,
}

fn route_to<'a>(target: SlotKey, scene: &Scene, buf: &'a mut [mem::MaybeUninit<SlotKey>]) -> &'a [SlotKey] {
	let mut count = 0;
	let mut current = target;
	while let Some(parent) = scene.parent(current) {
		if count >= buf.len() {
			panic!("Route path buffer overflow");
		}
		buf[count] = mem::MaybeUninit::new(current);
		count += 1;
		current = parent;
	}
	if count >= buf.len() {
		panic!("Route path buffer overflow");
	}
	buf[count] = mem::MaybeUninit::new(current);
	count += 1;
	unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const SlotKey, count) }
}

/// Routes an event to the target widget, starting from the root to the target.
fn route_event(ctx: &mut RouteEventContext, route: &[SlotKey], app: &mut dyn AppState, scene: &mut Scene) {
	let Some((&current, rest)) = route.split_last() else {
		return;
	};
	scene.with_widget(current, |widget, scene| {
		let scoped_app = app.scope_mut(current);
		let event_ctx = EventContext {
			time: ctx.time,
			target: ctx.target,
			bounds: ctx.bounds,
		};
		widget.event(ctx.event, &event_ctx, scene, scoped_app);
		if let Some(&child) = rest.last() {
			// Update the bounds for the child widget, ctx.bounds contains the bounds of the current widget.
			let local_bounds = widget.children().iter().find(|&slot| slot.key == child).unwrap().bounds;
			ctx.bounds = local_bounds + ctx.bounds.mins;
			route_event(ctx, rest, scoped_app, scene);
		}
		app.scope_exit(current);
	});
}

fn get_cursor_scoped<'a>(key: SlotKey, app: &'a dyn AppState, scene: &Scene) -> Option<(Option<Cursor>, &'a dyn AppState)> {
	let widget = scene.get_widget(key)?;
	let (parent_cursor, app) = if let Some(parent) = scene.parent(key) {
		get_cursor_scoped(parent, app, scene)?
	}
	else {
		(None, app)
	};
	let app = app.scope(key);
	Some((widget.cursor(app).or(parent_cursor), app))
}

fn layout_tree(key: SlotKey, ctx: &DrawContext, resx: &dyn Resources, mut app: &dyn AppState, scene: &mut Scene) {
	app = app.scope(key);
	scene.with_widget(key, |widget, scene| {
		widget.layout(ctx, resx, scene, app);

		for child in widget.children() {
			let child_bounds = child.bounds + ctx.bounds.mins;
			let child_ctx = DrawContext {
				viewport: ctx.viewport,
				clip: ctx.clip.intersect(child_bounds).unwrap_or_default(),
				time: ctx.time,
				bounds: child_bounds,
			};
			layout_tree(child.key, &child_ctx, resx, app, scene);
		}
	})
}

fn draw_tree<'a>(g: &mut Graphics, im: &mut im::DrawPool<'a>, key: SlotKey, ctx: &DrawContext, resx: &'a dyn Resources, mut app: &dyn AppState, scene: &mut Scene) {
	app = app.scope(key);
	scene.with_widget(key, |widget, scene| {
		widget.draw(g, im, ctx, resx, app);

		for child in widget.children() {
			let child_bounds = child.bounds + ctx.bounds.mins;
			let child_ctx = DrawContext {
				viewport: ctx.viewport,
				clip: ctx.clip.intersect(child_bounds).unwrap_or_default(),
				time: ctx.time,
				bounds: child_bounds,
			};
			draw_tree(g, im, child.key, &child_ctx, resx, app, scene);
		}
	})
}
