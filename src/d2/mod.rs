#![doc = include_str!("readme.md")]

use std::{cmp, fmt};
use cvmath::*;
use super::*;

mod paint;
mod pen;
mod scribe;
mod sprite;

mod color;
mod textured;

mod curve;

pub mod layout;

// Backwards compatibility re-exports
pub use im::{DrawBuffer, DrawBuilder, DrawPool};

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
