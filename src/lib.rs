pub extern crate cvmath;
use cvmath::prelude as cgmath;

mod graphics;
// mod canvas;
mod new_canvas;
#[macro_use]
mod shader;
mod vertex;
mod types;

pub use self::graphics::{IGraphics};
// pub use self::canvas::{Canvas, ICanvas};
pub use self::new_canvas::{Canvas, CanvasLock, ICanvas, VertexBuffer};
pub use self::shader::{Shader};
pub use self::vertex::{IVertex, PlaceV, Index};
pub use self::types::{Primitive, Blend, Visualize, Stencil};

// pub mod soft;
pub mod d2;
