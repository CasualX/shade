use super::*;

const DEFAULT_SCROLLBAR_WIDTH: i32 = 12;
const DEFAULT_SCROLLBAR_PADDING: i32 = 2;
const DEFAULT_MIN_THUMB_HEIGHT: i32 = 24;
const TRACK_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(24, 26, 31, 0.71));
const IDLE_THUMB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(112, 121, 137));
const ACTIVE_THUMB_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(174, 184, 198));
const SCROLL_STEP: i32 = 2;

/// Lightweight vertical scrollbar helper.
///
/// This is not a full widget yet; it owns only scrollbar-local state plus the
/// geometry, event, and draw helpers needed by scrollable containers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(super) struct ScrollVBar {
	width: i32,
	padding: i32,
	min_thumb_height: i32,
	scroll_y: i32,
	drag_offset: Option<i32>,
	hover_thumb: bool,
}

impl ScrollVBar {
	/// Creates a vertical scrollbar helper with the default GUI styling metrics.
	pub const fn new() -> ScrollVBar {
		ScrollVBar {
			width: DEFAULT_SCROLLBAR_WIDTH,
				padding: DEFAULT_SCROLLBAR_PADDING,
				min_thumb_height: DEFAULT_MIN_THUMB_HEIGHT,
				scroll_y: 0,
				drag_offset: None,
				hover_thumb: false,
			}
	}

	fn max_scroll(content_height: i32, viewport_height: i32) -> i32 {
		(content_height - viewport_height).max(0)
	}

	fn clamp_scroll(&mut self, viewport_height: i32, content_height: i32) {
		let max_scroll = Self::max_scroll(content_height, viewport_height);
		self.scroll_y = self.scroll_y.clamp(0, max_scroll);
	}

	/// Returns the current scroll offset.
	pub fn scroll_y(&self, viewport_height: i32, content_height: i32) -> i32 {
		self.scroll_y.clamp(0, Self::max_scroll(content_height, viewport_height))
	}

	/// Returns whether this scrollbar should be visible for the given content.
	pub fn visible(&self, viewport_size: cvmath::Vec2i, content_height: i32) -> bool {
		Self::max_scroll(content_height, viewport_size.y) > 0 && viewport_size.x > self.width
	}

	/// Returns the width this scrollbar reserves beside content.
	pub fn reserved_width(&self, viewport_size: cvmath::Vec2i, content_height: i32) -> i32 {
		if self.visible(viewport_size, content_height) {
			self.width
		}
		else {
			0
		}
	}

	/// Returns the local-space track rectangle.
	pub fn track(&self, viewport_size: cvmath::Vec2i, content_height: i32) -> Option<cvmath::Bounds2i> {
		if !self.visible(viewport_size, content_height) || viewport_size.y <= self.padding * 2 {
			return None;
		}
		Some(cvmath::Bounds2!(
			viewport_size.x - self.width + self.padding,
			self.padding,
			viewport_size.x - self.padding,
			viewport_size.y - self.padding
		))
	}

	/// Returns the local-space thumb rectangle for `scroll_y`.
	fn thumb(&self, viewport_size: cvmath::Vec2i, content_height: i32, scroll_y: i32) -> Option<cvmath::Bounds2i> {
		let track = self.track(viewport_size, content_height)?;
		let max_scroll = Self::max_scroll(content_height, viewport_size.y);
		if max_scroll == 0 {
			return None;
		}

		let track_height = track.height();
		let thumb_height = ((viewport_size.y as f32 / content_height.max(1) as f32) * track_height as f32).round() as i32;
		let thumb_height = thumb_height.clamp(self.min_thumb_height.min(track_height), track_height);
		let thumb_range = track_height - thumb_height;
		let thumb_top = if thumb_range == 0 {
			track.top()
		}
		else {
			track.top() + (scroll_y * thumb_range + max_scroll / 2) / max_scroll
		};
		Some(cvmath::Bounds2!(track.left(), thumb_top, track.right(), thumb_top + thumb_height))
	}

	/// Converts a thumb top position to a scroll offset.
	fn scroll_for_thumb_top(&self, viewport_size: cvmath::Vec2i, content_height: i32, scroll_y: i32, thumb_top: i32) -> i32 {
		let Some(track) = self.track(viewport_size, content_height) else {
			return 0;
		};
		let Some(thumb) = self.thumb(viewport_size, content_height, scroll_y) else {
			return 0;
		};
		let max_scroll = Self::max_scroll(content_height, viewport_size.y);
		let thumb_range = track.height() - thumb.height();
		if thumb_range <= 0 {
			return 0;
		}
		let top = thumb_top.clamp(track.top(), track.bottom() - thumb.height()) - track.top();
		(top * max_scroll + thumb_range / 2) / thumb_range
	}

	fn update_hover(&mut self, local: cvmath::Vec2i, viewport_size: cvmath::Vec2i, content_height: i32, scroll_y: i32) {
		self.hover_thumb = self.thumb(viewport_size, content_height, scroll_y).is_some_and(|thumb| thumb.contains(local));
	}

	/// Returns the cursor requested by the current scrollbar interaction state.
	pub fn cursor(&self) -> Option<Cursor> {
		if self.drag_offset.is_some() {
			Some(Cursor::Grabbing)
		}
		else if self.hover_thumb {
			Some(Cursor::Grab)
		}
		else {
			None
		}
	}

	/// Handles pointer input and returns a requested scroll offset when changed.
	fn pointer_event(&mut self, mouse: &MouseEvent, local: cvmath::Vec2i, viewport_size: cvmath::Vec2i, content_height: i32, owner: SlotKey, scene: &mut Scene) -> Option<i32> {
		let scroll_y = self.scroll_y(viewport_size.y, content_height);
		match mouse.kind {
			MouseEventKind::ButtonDown { button: MouseButton::LEFT } => {
				let track = self.track(viewport_size, content_height)?;
				if !track.contains(local) {
					self.hover_thumb = false;
					return None;
				}
				let thumb = self.thumb(viewport_size, content_height, scroll_y)?;
				self.hover_thumb = thumb.contains(local);
				if thumb.contains(local) {
					self.drag_offset = Some(local.y - thumb.top());
					scene.capture_pointer(owner);
					return None;
				}
				Some(self.scroll_for_thumb_top(viewport_size, content_height, scroll_y, local.y - thumb.height() / 2))
			},
			MouseEventKind::Move => {
				if let Some(offset) = self.drag_offset {
					Some(self.scroll_for_thumb_top(viewport_size, content_height, scroll_y, local.y - offset))
				}
				else {
					self.update_hover(local, viewport_size, content_height, scroll_y);
					None
				}
			},
			MouseEventKind::ButtonUp { button: MouseButton::LEFT } => {
				self.drag_offset = None;
				scene.release_pointer();
				self.update_hover(local, viewport_size, content_height, scroll_y);
				None
			},
			_ => None,
		}
	}

	/// Handles all scrollbar input.
	pub fn event(&mut self, mouse: &MouseEvent, ctx: &EventContext, owner: SlotKey, content_height: i32, scene: &mut Scene) {
		if matches!(mouse.kind, MouseEventKind::Leave) && self.drag_offset.is_none() {
			self.hover_thumb = false;
			return;
		}

		if !ctx.bounds.contains(mouse.pointer) && self.drag_offset.is_none() {
			self.hover_thumb = false;
			return;
		}

		let viewport_size = ctx.bounds.size();
		if let MouseEventKind::Wheel { delta } = mouse.kind {
			self.scroll_y -= delta * SCROLL_STEP;
			self.clamp_scroll(viewport_size.y, content_height);
			return;
		}

		let local = mouse.pointer - ctx.bounds.mins;
		if let Some(scroll_y) = self.pointer_event(mouse, local, viewport_size, content_height, owner, scene) {
			self.scroll_y = scroll_y;
			self.clamp_scroll(viewport_size.y, content_height);
		}
	}

	/// Clamps scrolling after viewport or content size changes.
	pub fn layout(&mut self, _time: time::Instant, viewport_size: cvmath::Vec2i, content_height: i32) {
		self.clamp_scroll(viewport_size.y, content_height);
	}

	/// Draws the scrollbar.
	pub fn draw<'a>(&self, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, content_height: i32) {
		let scroll_y = self.scroll_y(ctx.bounds.height(), content_height);
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		if let Some(track) = self.track(ctx.bounds.size(), content_height) {
			im.fill_rect(ctx, track, TRACK_COLOR, shader);
		}
		if let Some(thumb) = self.thumb(ctx.bounds.size(), content_height, scroll_y) {
			let color = if self.drag_offset.is_some() { ACTIVE_THUMB_COLOR } else { IDLE_THUMB_COLOR };
			im.fill_rect(ctx, thumb, color, shader);
		}
	}
}

impl Default for ScrollVBar {
	fn default() -> ScrollVBar {
		ScrollVBar::new()
	}
}
