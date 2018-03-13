/*!
Some 2D implementations.
*/

macro_rules! draw_primitive {
	(@count) => { 0 };
	(@count $e:expr $(, $tail:expr)*) => { 1 + draw_primitive!(@count $($tail),*) };

	($canvas:expr; $prim:expr; $($index:expr),*; $($vert:expr),*,) => {
		draw_primitive!($canvas; $prim; $($index),*; $($vert),*);
	};
	($canvas:expr; $prim:expr; $($index:expr),*; $($vert:expr),*) => {
		const N_VERTS: usize = draw_primitive!(@count $($vert),*);
		const N_INDICES: usize = draw_primitive!(@count $($index),*);
		assert_eq!(0, N_INDICES % $prim as u8 as usize);
		let (_vp, _ip) = $canvas.draw_primitive::<S>($prim, N_VERTS, N_INDICES / $prim as u8 as usize);
		debug_assert_eq!(_vp.len(), N_VERTS);
		debug_assert_eq!(_ip.len(), N_INDICES);
		let _i = -1isize;
		$(
			let _i = _i + 1;
			unsafe { *_ip.get_unchecked_mut(_i as usize) += $index; }
		)*
		let _v = -1isize;
		$(
			let _v = _v + 1;
			unsafe { *_vp.get_unchecked_mut(_v as usize) = $vert; }
		)*
	};
}

mod pen;
mod paint;
mod stamp;
mod vertex;
mod curve;
pub mod polygon;

pub use self::pen::{IPen, Pen};
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
