use super::*;

/// Graphics BaseObject handle.
///
/// Can be upcasted to specific object handles:
/// ```no_run
///# use todo as placeholder;
/// let g: shade::Graphics = placeholder!();
/// let base: shade::BaseObject = placeholder!();
/// let vbuf: shade::VertexBuffer = g.try_cast(base).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct BaseObject {
	pub(crate) value: u32,
}

impl BaseObject {
	pub const INVALID: BaseObject = BaseObject { value: 0 };

	/// Asserts that the handle is valid.
	#[inline]
	#[track_caller]
	pub fn unwrap(self) -> Self {
		#[cold]
		fn panic_invalid_handle(handle: BaseObject) -> ! {
			panic!("Invalid handle: BaseObject({})", handle.value);
		}
		if matches!(self, Self::INVALID) {
			panic_invalid_handle(self);
		}
		return self;
	}
}

macro_rules! define_handle {
	($name:ident) => {
		#[doc = concat!(stringify!($name), " handle.")]
		///
		/// Inherits [BaseObject](crate::BaseObject) (through `Into<BaseObject>`).
		#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
		#[repr(transparent)]
		pub struct $name {
			pub(crate) value: u32,
		}

		impl $name {
			/// Invalid handle.
			pub const INVALID: $name = $name { value: 0 };

			/// Asserts that the handle is valid.
			#[inline]
			#[track_caller]
			pub fn unwrap(self) -> Self {
				#[cold]
				fn panic_invalid_handle(handle: $name) -> ! {
					panic!(concat!("Invalid handle: ", stringify!($name),"({})"), handle.value);
				}
				if matches!(self, Self::INVALID) {
					panic_invalid_handle(self);
				}
				return self;
			}

			#[allow(dead_code)]
			#[inline]
			pub(crate) fn unwrap_or(self, default: Self) -> Self {
				if matches!(self, Self::INVALID) {
					return default;
				}
				return self;
			}
		}

		impl From<$name> for crate::BaseObject {
			#[inline]
			fn from($name { value }: $name) -> Self {
				crate::BaseObject { value }
			}
		}
	};
}

/// Types of graphics objects.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectType {
	/// Vertex buffer.
	VertexBuffer,
	/// Index buffer.
	IndexBuffer,
	/// Shader program.
	ShaderProgram,
	/// Texture 2D.
	Texture2D,
}

/// Trait for casting object handles to specific object types.
pub trait ObjectCast<U> {
	/// Implementation of [Graphics::try_cast].
	fn try_cast(self, g: &Graphics) -> Option<U>;
}

impl ObjectCast<BaseObject> for BaseObject {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<BaseObject> {
		Some(self)
	}
}
impl ObjectCast<VertexBuffer> for BaseObject {
	#[inline]
	fn try_cast(self, g: &Graphics) -> Option<VertexBuffer> {
		match g.get_type(self)? {
			ObjectType::VertexBuffer => Some(VertexBuffer { value: self.value }),
			_ => None,
		}
	}
}
impl ObjectCast<IndexBuffer> for BaseObject {
	#[inline]
	fn try_cast(self, g: &Graphics) -> Option<IndexBuffer> {
		match g.get_type(self)? {
			ObjectType::IndexBuffer => Some(IndexBuffer { value: self.value }),
			_ => None,
		}
	}
}
impl ObjectCast<ShaderProgram> for BaseObject {
	#[inline]
	fn try_cast(self, g: &Graphics) -> Option<ShaderProgram> {
		match g.get_type(self)? {
			ObjectType::ShaderProgram => Some(ShaderProgram { value: self.value }),
			_ => None,
		}
	}
}
impl ObjectCast<Texture2D> for BaseObject {
	#[inline]
	fn try_cast(self, g: &Graphics) -> Option<Texture2D> {
		match g.get_type(self)? {
			ObjectType::Texture2D => Some(Texture2D { value: self.value }),
			_ => None,
		}
	}
}

impl ObjectCast<BaseObject> for VertexBuffer {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<BaseObject> {
		Some(BaseObject { value: self.value })
	}
}
impl ObjectCast<BaseObject> for IndexBuffer {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<BaseObject> {
		Some(BaseObject { value: self.value })
	}
}
impl ObjectCast<BaseObject> for ShaderProgram {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<BaseObject> {
		Some(BaseObject { value: self.value })
	}
}
impl ObjectCast<BaseObject> for Texture2D {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<BaseObject> {
		Some(BaseObject { value: self.value })
	}
}

impl ObjectCast<VertexBuffer> for VertexBuffer {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<VertexBuffer> {
		Some(self)
	}
}
impl ObjectCast<IndexBuffer> for IndexBuffer {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<IndexBuffer> {
		Some(self)
	}
}
impl ObjectCast<ShaderProgram> for ShaderProgram {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<ShaderProgram> {
		Some(self)
	}
}
impl ObjectCast<Texture2D> for Texture2D {
	#[inline]
	fn try_cast(self, _g: &Graphics) -> Option<Texture2D> {
		Some(self)
	}
}
