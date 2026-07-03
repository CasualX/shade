use super::*;

/// Context for drawing.
pub struct DrawContext {
	/// The bounds used for root-space projection.
	pub viewport: cvmath::Bounds2i,
	/// The clipping rectangle for the current draw operation.
	pub clip: cvmath::Bounds2i,
	/// The current time for the draw operation.
	pub time: time::Instant,
	/// The widget bounds in scene coordinates.
	pub bounds: cvmath::Bounds2i,
}

impl DrawContext {
	/// Converts the top-left-origin GUI clip rectangle to a bottom-left-origin graphics scissor.
	pub fn scissor(&self) -> cvmath::Bounds2i {
		let Some(clip) = self.clip.intersect(self.viewport) else {
			return cvmath::Bounds2i::vec(cvmath::Vec2i::ZERO);
		};
		let y_min = self.viewport.mins.y + self.viewport.maxs.y - clip.maxs.y;
		let y_max = self.viewport.mins.y + self.viewport.maxs.y - clip.mins.y;
		cvmath::Bounds2i::new(cvmath::Point2(clip.mins.x, y_min), cvmath::Point2(clip.maxs.x, y_max))
	}
}

#[allow(unused_variables)]
/// Common interface implemented by every retained GUI widget.
pub trait Widget: any::Any {
	/// Gets the unique identifier for this widget.
	fn key(&self) -> SlotKey;

	/// Gets the preferred cursor when the widget is the current target.
	/// Returning `None` inherits the nearest ancestor cursor.
	fn cursor(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> Option<Cursor> { None }

	/// Gets the direct children of this widget.
	fn children(&self) -> &[widgets::ChildWidget] { &[] }

	/// Returns whether this widget is interactive.
	fn hittable(&self) -> bool { true }

	/// Returns whether this top-level widget can be dragged by the root panel.
	fn draggable(&self) -> bool { false }

	/// Handles an event.
	fn event(&mut self, event: &InputEvent, event_ctx: &EventContext, scene: &mut Scene, app: &mut dyn AppState, app_ctx: &mut dyn AppContext) {}

	/// Run layout before drawing.
	fn layout(&mut self, ctx: &DrawContext, resx: &dyn Resources, scene: &mut Scene, app: &dyn AppState, app_ctx: &dyn AppContext) {}

	/// Draws the widget itself.
	fn draw<'a>(&mut self, g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState, app_ctx: &dyn AppContext) {}
}

impl dyn Widget {
	/// Downcasts this widget by concrete type.
	pub fn downcast_ref<T: Widget>(&self) -> Option<&T> {
		(self as &dyn any::Any).downcast_ref::<T>()
	}

	/// Downcasts this widget mutably by concrete type.
	pub fn downcast_mut<T: Widget>(&mut self) -> Option<&mut T> {
		(self as &mut dyn any::Any).downcast_mut::<T>()
	}
}

impl dto::Widget {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		match self {
			dto::Widget::Button(dto) => dto.construct(scene, ctx),
			dto::Widget::Checkbox(dto) => dto.construct(scene, ctx),
			dto::Widget::ColorSwatch(dto) => dto.construct(scene, ctx),
			dto::Widget::DrawGrid(dto) => dto.construct(scene, ctx),
			dto::Widget::Label(dto) => dto.construct(scene, ctx),
			dto::Widget::Panel(dto) => dto.construct(scene, ctx),
			dto::Widget::ProgressBar(dto) => dto.construct(scene, ctx),
			dto::Widget::RadioButton(dto) => dto.construct(scene, ctx),
			dto::Widget::ScrollPanel(dto) => dto.construct(scene, ctx),
			dto::Widget::Slider(dto) => dto.construct(scene, ctx),
			dto::Widget::Window(dto) => dto.construct(scene, ctx),
		}
	}
}
