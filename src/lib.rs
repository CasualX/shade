pub extern crate cvmath;
use cvmath::prelude as cgmath;

mod graphics;
mod canvas;
mod shader;
mod vertex;
mod types;
mod uniform;
mod traits;

pub use self::graphics::{IGraphics};
pub use self::canvas::{Canvas, ICanvas};
pub use self::shader::{TShader};
pub use self::vertex::{TVertex, PlaceV, Index};
pub use self::types::{Primitive, Blend, Visualize, Stencil};
pub use self::traits::{Allocate};
pub use self::uniform::{TUniform};

pub mod soft;
pub mod d2;
