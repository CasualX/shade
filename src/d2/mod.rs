/*!
Some 2D implementations.
*/

mod pen;
mod paint;
mod stamp;
mod vertex;
mod curve;
pub mod polygon;

pub use self::pen::{DrawPath, IPen, Pen};
pub use self::paint::{IPaint, Paint};
pub use self::stamp::{IStamp, Stamp};
pub use self::vertex::{TexV, ColorV, TextV, ToVertex};
pub use self::curve::{bezier2, bezier3};

pub type Point2 = ::cgmath::Point2<f32>;
pub type Vec2 = ::cgmath::Vec2<f32>;
pub type Rect = ::cgmath::Rect<f32>;
pub type Rad = ::cgmath::Rad<f32>;
pub type Deg = ::cgmath::Deg<f32>;
pub type Affine2 = ::cgmath::Affine2<f32>;
pub type Color = ::cgmath::Vec4<f32>;

#[allow(non_snake_case)]
pub fn Point2(x: f32, y: f32) -> Point2 { Point2 { x, y } }
#[allow(non_snake_case)]
pub fn Vec2(x: f32, y: f32) -> Vec2 { Vec2 { x, y } }
#[allow(non_snake_case)]
pub fn Rect(mins: Point2, maxs: Point2) -> Rect { Rect { mins, maxs } }
#[allow(non_snake_case)]
pub fn Rad(rad: f32) -> Rad { ::cgmath::Rad(rad) }
#[allow(non_snake_case)]
pub fn Deg(deg: f32) -> Deg { ::cgmath::Deg(deg) }
#[allow(non_snake_case)]
pub fn Color(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
	Color { x: red, y: green, z: blue, w: alpha }
}

#[cfg(test)]
mod tests;
