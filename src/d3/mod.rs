//! Graphics module 3D.

use super::*;
use cvmath::*;

#[macro_use]
mod macros;

mod camera;
pub use self::camera::*;

pub mod color3d;
pub mod submesh;

pub mod axes;
pub mod frustum;
