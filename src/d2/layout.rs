//! Layout utilities.

/// Specifies layout direction.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Orientation {
	Horizontal,
	Vertical,
}

impl Orientation {
	pub const fn flip(self) -> Self {
		match self {
			Self::Horizontal => Self::Vertical,
			Self::Vertical => Self::Horizontal,
		}
	}
}

/// Unit of measurement.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Unit {
	/// An absolute size.
	Abs(f32),
	/// A percentage of the total size.
	Pct(f32),
	/// A fraction of the remaining space.
	Fr(f32),
}

/// Justification method.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Justify {
	/// Align spans to the start of the range.
	Start,
	/// Center spans within the range.
	Center,
	/// Align spans to the end of the range.
	End,
	/// Distribute spans evenly with equal space between them.
	SpaceBetween,
	/// Distribute spans with equal space around them.
	SpaceAround,
	/// Distribute spans with equal space between them and at the ends.
	SpaceEvenly,
}

mod flex;
pub use flex::*;
