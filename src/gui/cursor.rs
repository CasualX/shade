
/// Various cursor types.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cursor {
	/// Use the platform default cursor.
	Default,
	/// Indicates a clickable target.
	Pointer,
	/// Indicates a draggable target.
	Grab,
	/// Indicates an active drag operation.
	Grabbing,
	/// Indicates crosshair-style precision targeting.
	Crosshair,
	/// Indicates freeform movement.
	Move,
	/// Indicates horizontal resizing.
	ResizeHorizontal,
	/// Indicates vertical resizing.
	ResizeVertical,
	/// Indicates resizing along the northwest-southeast diagonal.
	ResizeNwse,
	/// Indicates resizing along the northeast-southwest diagonal.
	ResizeNesw,
}
