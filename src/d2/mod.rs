#![doc = include_str!("readme.md")]

use std::{cmp, fmt};
use cvmath::*;
use super::*;

mod paint;
mod pen;
mod post_process;
mod scribe;
mod sprite;

mod color;
mod textured;

mod curve;

pub mod layout;
pub mod polygon;

// Backwards compatibility re-exports
pub use im::{DrawBuffer, DrawBuilder, DrawPool};

pub use self::color::*;
pub use self::textured::*;
pub use self::paint::Paint;
pub use self::pen::Pen;
pub use self::scribe::*;
pub use self::sprite::Sprite;
pub use self::post_process::PostProcessQuad;

/// Generate vertex data from a template.
pub trait ToVertex<V> {
	fn to_vertex(&self, pos: Point2f, index: usize) -> V;
}

#[cfg(test)]
mod tests;
