use std::{mem, ops, slice, time};

/// Re-export of compatible `cvmath` crate.
pub use cvmath;

#[macro_use]
mod handle;

mod common;
mod graphics;
mod vertex;
mod texture;
mod uniform;
mod shader;
mod resources;
mod sstring;

pub use self::common::*;
pub use self::graphics::*;
pub use self::vertex::*;
pub use self::texture::*;
pub use self::uniform::*;
pub use self::shader::Shader;

pub mod d2;
pub mod d3;
pub mod im;

pub mod color;
pub mod image;

#[cfg(feature = "gl")]
pub mod gl;

#[cfg(feature = "webgl")]
pub mod webgl;

#[cfg(feature = "msdfgen")]
pub mod msdfgen;
