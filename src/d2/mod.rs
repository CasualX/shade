/*!
2D graphics.
*/

use std::{cmp, fmt};
use super::*;
use cvmath::*;

mod cmdbuf;
mod paint;
mod pen;
mod stamp;
mod curve;
mod scribe;
pub mod layout;

pub use self::cmdbuf::{CommandBuffer, PrimBuilder};
pub use self::paint::Paint;
pub use self::pen::Pen;
pub use self::stamp::Stamp;
pub use self::scribe::*;

/// Generate vertex data from a template.
pub trait ToVertex<V> {
	fn to_vertex(&self, pos: Point2<f32>, index: usize) -> V;
}

#[cfg(test)]
mod tests;
