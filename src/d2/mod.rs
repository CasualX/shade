#![doc = include_str!("readme.md")]

use std::{cmp, fmt};
use cvmath::*;
use super::*;

mod drawbuf;
mod pool;

mod paint;
mod pen;
mod scribe;
mod sprite;

mod color;
mod textured;

mod curve;

pub mod layout;

pub use self::drawbuf::{DrawCommand, DrawBuffer, DrawBuilder, PipelineState, PrimBuilder};
pub use self::pool::DrawPool;
pub use self::color::*;
pub use self::textured::*;
pub use self::paint::Paint;
pub use self::pen::Pen;
pub use self::sprite::Sprite;
pub use self::scribe::*;

/// Generate vertex data from a template.
pub trait ToVertex<V> {
	fn to_vertex(&self, pos: Point2f, index: usize) -> V;
}

#[cfg(test)]
mod tests;
