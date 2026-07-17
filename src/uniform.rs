use std::{any, slice};

/// Values that can be passed as shader uniforms.
pub trait TUniformValue {
	fn set(&self, name: &str, set: &mut dyn UniformSetter);
}

/// Indirection over a backend-specific uniform setter.
pub trait UniformSetter {
	fn float(&mut self, name: &str, data: &[f32]);
	fn vec2(&mut self, name: &str, data: &[cvmath::Vec2<f32>]);
	fn vec3(&mut self, name: &str, data: &[cvmath::Vec3<f32>]);
	fn vec4(&mut self, name: &str, data: &[cvmath::Vec4<f32>]);

	fn int(&mut self, name: &str, data: &[i32]);
	fn ivec2(&mut self, name: &str, data: &[cvmath::Vec2<i32>]);
	fn ivec3(&mut self, name: &str, data: &[cvmath::Vec3<i32>]);
	fn ivec4(&mut self, name: &str, data: &[cvmath::Vec4<i32>]);

	fn mat2(&mut self, name: &str, data: &[cvmath::Mat2f]);
	fn mat3(&mut self, name: &str, data: &[cvmath::Mat3f]);
	fn mat4(&mut self, name: &str, data: &[cvmath::Mat4f]);

	fn transform2(&mut self, name: &str, data: &[cvmath::Transform2f]);
	fn transform3(&mut self, name: &str, data: &[cvmath::Transform3f]);

	fn sampler2d(&mut self, name: &str, texture: &[&dyn crate::Texture2D]);
}

impl dyn UniformSetter + '_ {
	/// Sets a uniform uniform value.
	#[inline]
	pub fn value<T: TUniformValue + ?Sized>(&mut self, name: &str, value: &T) {
		value.set(name, self)
	}
}

/// Marker trait for types containing uniform values.
pub trait TUniform: Clone + PartialEq + Default + UniformVisitor {}

/// Provides a stable type identifier for a uniform type.
///
/// Uniform types containing lifetimes should return the [`any::TypeId`] of an
/// equivalent type with those lifetimes replaced by `'static`. Uniform types
/// without lifetimes can return their own [`any::TypeId`].
///
/// # Safety
///
/// The returned identifier must uniquely identify the concrete layout and type
/// of `Self`, ignoring lifetimes only. Unsafe code may use matching identifiers
/// as proof that erased pointers have the same concrete type and transmute or
/// cast between them. Returning the same identifier for incompatible types can
/// therefore cause undefined behavior.
pub unsafe trait TUniformKey {
	fn key() -> any::TypeId;
}

impl<T: Clone + PartialEq + Default + UniformVisitor> TUniform for T {}

/// Visiting an instance and pushing its uniforms.
///
/// Useful for bulk uniform submission via a visitor.
pub trait UniformVisitor {
	fn visit(&self, set: &mut dyn UniformSetter);
}

/// Uniform visitor from closure.
pub struct UniformFn<F: Fn(&mut dyn UniformSetter)>(pub F);

impl<'a, F: Fn(&mut dyn UniformSetter)> UniformVisitor for UniformFn<F> {
	#[inline]
	fn visit(&self, set: &mut dyn UniformSetter) {
		(self.0)(set);
	}
}

macro_rules! impl_tuniform_value {
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
		impl<const N: usize> TUniformValue for [$ty; N] {
			#[inline]
			fn set(&self, name: &str, set: &mut dyn UniformSetter) {
				set.$set_fn(name, self);
			}
		}
	};
}

impl_tuniform_value!(f32, float);
impl_tuniform_value!(i32, int);

impl_tuniform_value!(cvmath::Vec2<f32>, vec2);
impl_tuniform_value!(cvmath::Vec3<f32>, vec3);
impl_tuniform_value!(cvmath::Vec4<f32>, vec4);

impl_tuniform_value!(cvmath::Vec2<i32>, ivec2);
impl_tuniform_value!(cvmath::Vec3<i32>, ivec3);
impl_tuniform_value!(cvmath::Vec4<i32>, ivec4);

impl TUniformValue for dyn crate::Texture2D + '_ {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, slice::from_ref(&self));
	}
}
impl TUniformValue for &(dyn crate::Texture2D + '_) {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, slice::from_ref(self));
	}
}
impl TUniformValue for [&(dyn crate::Texture2D + '_)] {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, self);
	}
}
impl<const N: usize> TUniformValue for [&(dyn crate::Texture2D + '_); N] {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.sampler2d(name, self);
	}
}

impl_tuniform_value!(cvmath::Mat2f, mat2);
impl_tuniform_value!(cvmath::Mat3f, mat3);
impl_tuniform_value!(cvmath::Mat4f, mat4);

impl_tuniform_value!(cvmath::Transform2f, transform2);
impl_tuniform_value!(cvmath::Transform3f, transform3);

impl TUniformValue for cvmath::Bounds2<i32> {
	#[inline]
	fn set(&self, name: &str, set: &mut dyn UniformSetter) {
		set.ivec2(name, AsRef::as_ref(self));
	}
}
