use super::*;

/// Key to look up resources.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct ResourceKey {
	value: u32,
}

/// ResourceKey constructor.
#[inline]
#[allow(non_snake_case)]
pub const fn ResourceKey(value: u32) -> ResourceKey {
	ResourceKey { value }
}

impl ResourceKey {
	/// Returns the underlying value of the key.
	#[inline]
	pub const fn value(&self) -> u32 {
		self.value
	}
}

/// Trait for looking up resources by key.
#[allow(unused_variables)]
pub trait Resources {
	/// Gets a resource by type.
	#[inline]
	fn get_any(&self, ty: any::TypeId) -> Option<&dyn any::Any> { None }

	/// Gets a FontResource by key.
	#[inline]
	fn get_font<'a>(&'a self, key: ResourceKey) -> Option<d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>> { None }

	/// Gets a ShaderProgram by key.
	#[inline]
	fn get_shader<'a>(&'a self, key: ResourceKey) -> Option<&'a dyn ShaderProgram> { None }

	/// Gets a Texture2D by key.
	#[inline]
	fn get_texture2d<'a>(&'a self, key: ResourceKey) -> Option<&'a dyn Texture2D> { None }
}

impl dyn Resources + '_ {
	/// Gets a resource by type.
	#[inline]
	pub fn get_custom<T: 'static>(&self) -> Option<&T> {
		self.get_any(any::TypeId::of::<T>()).and_then(|r| r.downcast_ref::<T>())
	}
}

/// System resources available for drawing.
pub struct SystemResources<'a> {
	/// Font resource used by text-drawing helpers and widgets.
	pub font: d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>,
	/// Shader used for flat-color GUI geometry.
	pub color_shader: &'a dyn ShaderProgram,
}

impl SystemResources<'_> {
	pub const FONT_KEY: ResourceKey = ResourceKey(0);
	pub const COLOR_SHADER_KEY: ResourceKey = ResourceKey(1);
}

impl<'a> Resources for SystemResources<'a> {
	fn get_font(&self, key: ResourceKey) -> Option<d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>> {
		match key {
			Self::FONT_KEY => Some(self.font),
			_ => None,
		}
	}

	fn get_shader(&self, key: ResourceKey) -> Option<&'a dyn ShaderProgram> {
		match key {
			Self::COLOR_SHADER_KEY => Some(self.color_shader),
			_ => None,
		}
	}
}
