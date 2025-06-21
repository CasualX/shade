use std::slice;

use crate::Texture2D;

pub trait TUniformValue {
	fn set(&self, name: &str, set: &mut dyn UniformSetter);
}

pub trait UniformSetter {
	fn d1v(&mut self, name: &str, data: &[f64]);
	fn d2v(&mut self, name: &str, data: &[[f64; 2]]);
	fn d3v(&mut self, name: &str, data: &[[f64; 3]]);
	fn d4v(&mut self, name: &str, data: &[[f64; 4]]);
	fn f1v(&mut self, name: &str, data: &[f32]);
	fn f2v(&mut self, name: &str, data: &[[f32; 2]]);
	fn f3v(&mut self, name: &str, data: &[[f32; 3]]);
	fn f4v(&mut self, name: &str, data: &[[f32; 4]]);
	fn i1v(&mut self, name: &str, data: &[i32]);
	fn i2v(&mut self, name: &str, data: &[[i32; 2]]);
	fn i3v(&mut self, name: &str, data: &[[i32; 3]]);
	fn i4v(&mut self, name: &str, data: &[[i32; 4]]);
	fn mat2(&mut self, name: &str, data: &[cvmath::Mat2f]);
	fn mat3(&mut self, name: &str, data: &[cvmath::Mat3f]);
	fn mat4(&mut self, name: &str, data: &[cvmath::Mat4f]);
	fn transform2(&mut self, name: &str, data: &[cvmath::Transform2f]);
	fn transform3(&mut self, name: &str, data: &[cvmath::Transform3f]);
	fn sampler2d(&mut self, name: &str, texture: &[Texture2D]);
}

impl<'a> dyn UniformSetter + 'a {
	#[inline]
	pub fn value<T: TUniformValue + ?Sized>(&mut self, name: &str, value: &T) {
		value.set(name, self)
	}
}

/// Visiting uniform parameters.
///
/// Uniform structs should be `Clone` and `Debug`.
pub trait UniformVisitor {
	fn visit(&self, f: &mut dyn UniformSetter);
}

macro_rules! impl_uniform_value {
	($ty:ty, $set1_fn:ident, $set2_fn:ident, $set3_fn:ident, $set4_fn:ident) => {
		impl TUniformValue for $ty {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set1_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty; 2] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set2_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty; 3] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set3_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty; 4] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set4_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set1_fn(name, self);
			}
		}
		impl TUniformValue for [[$ty; 2]] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set2_fn(name, self);
			}
		}
		impl TUniformValue for [[$ty; 3]] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set3_fn(name, self);
			}
		}
		impl TUniformValue for [[$ty; 4]] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set4_fn(name, self);
			}
		}
		impl TUniformValue for cvmath::Vec2<$ty> {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set2_fn(name, slice::from_ref(self.as_ref()));
			}
		}
		impl TUniformValue for cvmath::Vec3<$ty> {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set3_fn(name, slice::from_ref(self.as_ref()));
			}
		}
		impl TUniformValue for cvmath::Vec4<$ty> {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set4_fn(name, slice::from_ref(self.as_ref()));
			}
		}
		impl TUniformValue for cvmath::Bounds2<$ty> {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				let data: &[[$ty; 2]] = unsafe { slice::from_raw_parts(&*(self as *const _ as *const [$ty; 2]), 2) };
				set.$set2_fn(name, data);
			}
		}
	};
}

impl_uniform_value!(f64, d1v, d2v, d3v, d4v);
impl_uniform_value!(f32, f1v, f2v, f3v, f4v);
impl_uniform_value!(i32, i1v, i2v, i3v, i4v);
// impl_uniform_value!(bool, v1b, v2b, v3b, v4b);

impl TUniformValue for crate::Texture2D {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, slice::from_ref(self));
	}
}
impl TUniformValue for [crate::Texture2D] {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, self);
	}
}

macro_rules! impl_uniform_matrix {
	($ty:ty, $layout:expr, $set_fn:ident) => {
		impl TUniformValue for $ty {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set_fn(name, self);
			}
		}
	}
}

impl_uniform_matrix!(cvmath::Mat2f, MatrixLayout::RowMajor, mat2);
impl_uniform_matrix!(cvmath::Mat3f, MatrixLayout::RowMajor, mat3);
impl_uniform_matrix!(cvmath::Mat4f, MatrixLayout::RowMajor, mat4);

macro_rules! impl_transform_matrix {
	($ty:ty, $set_fn:ident) => {
		impl TUniformValue for $ty {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set_fn(name, slice::from_ref(self));
			}
		}
		impl TUniformValue for [$ty] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set_fn(name, self);
			}
		}
	};
}

impl_transform_matrix!(cvmath::Transform2f, transform2);
impl_transform_matrix!(cvmath::Transform3f, transform3);
