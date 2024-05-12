use std::{mem, ops};

#[macro_use]
mod handle;

mod common;
mod graphics;
mod buffer;
mod vertex;
mod texture;
mod surface;
mod uniform;
mod shader;
mod resources;

use handle::Handle;
use resources::{Resource, ResourceMap};

pub use self::common::{PrimType, BlendMode, DepthTest, CullMode, BufferUsage};
pub use self::graphics::{IGraphics, Graphics, GfxError, ClearArgs, DrawArgs, DrawIndexedArgs};
pub use self::buffer::{VertexBuffer, IndexBuffer};
pub use self::vertex::{TVertex, VertexAttributeFormat, VertexAttribute, VertexLayout};
pub use self::texture::{Texture2D, TextureFormat, TextureWrap, TextureFilter, Texture2DInfo};
pub use self::surface::{Surface, SurfaceFormat, SurfaceInfo};
pub use self::uniform::{UniformBuffer, TUniform, UniformLayout, UniformAttribute, UniformMatOrder, UniformType};
pub use self::shader::Shader;

pub mod d2;
pub mod gl;

#[cfg(feature = "png")]
pub mod png;
