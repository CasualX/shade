/*!
2D graphics.
*/

use std::cmp;
use super::*;
use cvmath::*;

mod canvas;
mod paint;
mod pen;
mod stamp;
mod curve;

pub use self::canvas::{Canvas, PrimBuilder};
pub use self::paint::Paint;
pub use self::pen::Pen;
pub use self::stamp::Stamp;

/// Generate vertex data from a template.
pub trait ToVertex<V> {
	fn to_vertex(&self, pos: Point2<f32>, index: usize) -> V;
}

#[cfg(test)]
mod tests;
