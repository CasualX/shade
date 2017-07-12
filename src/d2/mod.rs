/*!
Some 2D implementations.
*/

mod pen;
mod paint;
mod stamp;
mod vertex;
mod curve;

pub use self::pen::{DrawPath, IPen, Pen};
pub use self::paint::{IPaint, Paint};
pub use self::stamp::{IStamp, Stamp};
pub use self::vertex::{TexV, ColorV, TextV, ToVertex};
pub use self::curve::{bezier2, bezier3};

pub type Point = ::cgmath::Point2<f32>;
pub type Vec2 = ::cgmath::Vec2<f32>;
pub type Rect = ::cgmath::Rect<f32>;
pub type Rad = ::cgmath::Rad<f32>;
pub type Deg = ::cgmath::Deg<f32>;
pub type Affine = ::cgmath::Affine2<f32>;
pub type Color = ::cgmath::Vec4<f32>;

#[cfg(test)]
mod tests;
