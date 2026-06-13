use super::*;

/// Kinds of mouse input handled by the GUI.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseEventKind {
	/// The cursor entered this widget's pointed route.
	Enter,
	/// The cursor left this widget's pointed route.
	Leave,
	/// The cursor moved.
	Move,
	/// A mouse button was pressed.
	ButtonDown { button: MouseButton },
	/// A mouse button was released.
	ButtonUp { button: MouseButton },
	/// The mouse wheel moved.
	Wheel { delta: i32 },
}

/// Mouse input delivered to the GUI scene.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
	/// The mouse action that occurred.
	pub kind: MouseEventKind,
	/// Cursor position in scene coordinates.
	pub pointer: cvmath::Vec2i,
}

/// Raw input delivered to widgets in the GUI tree.
#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
	Mouse(MouseEvent),
}

impl InputEvent {
	/// Returns the mouse payload when this input event is mouse-driven.
	pub fn mouse(&self) -> Option<&MouseEvent> {
		match self {
			InputEvent::Mouse(event) => Some(event),
		}
	}
}

/// Identifier for a mouse button.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MouseButton(pub u8);

impl MouseButton {
	pub const LEFT: MouseButton = MouseButton(0);
	pub const RIGHT: MouseButton = MouseButton(1);
	pub const MIDDLE: MouseButton = MouseButton(2);
}

/// Context for event handling.
pub struct EventContext {
	/// The current time for the event.
	pub time: time::Instant,
	/// Target of the event.
	pub target: SlotKey,
	/// The widget bounds in scene coordinates.
	pub bounds: cvmath::Bounds2i,
}
