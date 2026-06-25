//! Built-in retained GUI widgets.

use super::*;

mod button;
mod checkbox;
mod color_swatch;
mod draw_grid;
mod label;
mod menu;
mod menu_bar;
mod menu_bar_item;
mod menu_item;
mod panel;
mod progress_bar;
mod radio_button;
mod root_panel;
mod scroll_bar;
mod scroll_panel;
mod separator;
mod slider;
mod window;

pub use self::button::*;
pub use self::checkbox::*;
pub use self::color_swatch::*;
pub use self::draw_grid::*;
pub use self::label::*;
pub use self::menu::*;
pub use self::menu_bar::*;
pub use self::menu_bar_item::*;
pub use self::menu_item::*;
pub use self::panel::*;
pub use self::progress_bar::*;
pub use self::radio_button::*;
pub(super) use self::root_panel::*;
use self::scroll_bar::*;
pub use self::scroll_panel::*;
pub use self::separator::*;
pub use self::slider::*;
pub use self::window::*;

/// A widget child positioned by its parent.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChildWidget {
	/// Bounds relative to the parent widget.
	pub bounds: cvmath::Bounds2i,
	/// Key of the child widget.
	pub key: SlotKey,
}

impl ChildWidget {
	/// Creates a positioned child widget.
	pub const fn new(bounds: cvmath::Bounds2i, key: SlotKey) -> ChildWidget {
		ChildWidget { bounds, key }
	}
}
