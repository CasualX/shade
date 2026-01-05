//! Graphics module 3D.

use super::*;
use cvmath::*;

mod camera;
pub use self::camera::*;

mod textured;
pub use self::textured::*;

mod color3d;
pub use self::color3d::*;

mod mesh;
pub use self::mesh::*;

pub mod submesh;

pub mod axes;
pub mod frustum;
pub mod icosahedron;
