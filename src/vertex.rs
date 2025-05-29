
/// Defines a vertex type.
pub unsafe trait TVertex: Copy + Default + dataview::Pod {
	const LAYOUT: &'static VertexLayout;
}

/// VertexAttribute size.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum VertexAttributeSize {
	/// 1 component.
	One = 1,
	/// 2 components.
	Two = 2,
	/// 3 components.
	Three = 3,
	/// 4 components.
	Four = 4,
}

/// VertexAttribute type.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum VertexAttributeType {
	F32,
	F64,
	I32,
	U32,
	I16,
	U16,
	I8,
	U8,
}

/// VertexAttribute format.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum VertexAttributeFormat {
	/// 32-bit floating point number.
	F32,
	/// 32-bit floating point number, 2 components.
	F32v2,
	/// 32-bit floating point number, 3 components.
	F32v3,
	/// 32-bit floating point number, 4 components.
	F32v4,

	/// 64-bit floating point number.
	F64,
	/// 64-bit floating point number, 2 components.
	F64v2,
	/// 64-bit floating point number, 3 components.
	F64v3,
	/// 64-bit floating point number, 4 components.
	F64v4,

	/// 32-bit signed integer.
	I32,
	/// 32-bit signed integer, 2 components.
	I32v2,
	/// 32-bit signed integer, 3 components.
	I32v3,
	/// 32-bit signed integer, 4 components.
	I32v4,

	/// 32-bit unsigned integer.
	U32,
	/// 32-bit unsigned integer, 2 components.
	U32v2,
	/// 32-bit unsigned integer, 3 components.
	U32v3,
	/// 32-bit unsigned integer, 4 components.
	U32v4,

	/// 16-bit signed integer.
	I16,
	/// 16-bit signed integer, 2 components.
	I16v2,
	/// 16-bit signed integer, 3 components.
	I16v3,
	/// 16-bit signed integer, 4 components.
	I16v4,

	/// 16-bit unsigned integer.
	U16,
	/// 16-bit unsigned integer, 2 components.
	U16v2,
	/// 16-bit unsigned integer, 3 components.
	U16v3,
	/// 16-bit unsigned integer, 4 components.
	U16v4,

	/// 8-bit signed integer.
	I8,
	/// 8-bit signed integer, 2 components.
	I8v2,
	/// 8-bit signed integer, 3 components.
	I8v3,
	/// 8-bit signed integer, 4 components.
	I8v4,

	/// 8-bit unsigned integer.
	U8,
	/// 8-bit unsigned integer, 2 components.
	U8v2,
	/// 8-bit unsigned integer, 3 components.
	U8v3,
	/// 8-bit unsigned integer, 4 components.
	U8v4,

	/// Normalized `i16`.
	///
	/// The value is mapped from the range `[-32768, 32767]` to `[-1.0, 1.0]`.
	I16Norm,
	/// Normalized `u16`, 2 components.
	I16Normv2,
	/// Normalized `u16`, 3 components.
	I16Normv3,
	/// Normalized `u16`, 4 components.
	I16Normv4,

	/// Normalized `u16`.
	///
	/// The value is mapped from the range `[0, 65535]` to `[0.0, 1.0]`.
	U16Norm,
	/// Normalized `u16`, 2 components.
	U16Normv2,
	/// Normalized `u16`, 3 components.
	U16Normv3,
	/// Normalized `u16`, 4 components.
	U16Normv4,

	/// Normalized `i8`.
	///
	/// The value is mapped from the range `[-128, 127]` to `[-1.0, 1.0]`.
	I8Norm,
	/// Normalized `i8`, 2 components.
	I8Normv2,
	/// Normalized `i8`, 3 components.
	I8Normv3,
	/// Normalized `i8`, 4 components.
	I8Normv4,

	/// Normalized `u8`.
	///
	/// The value is mapped from the range `[0, 255]` to `[0.0, 1.0]`.
	U8Norm,
	/// Normalized `u8`, 2 components.
	U8Normv2,
	/// Normalized `u8`, 3 components.
	U8Normv3,
	/// Normalized `u8`, 4 components.
	U8Normv4,
}

impl VertexAttributeFormat {
	/// Returns the size of the vertex attribute.
	#[inline]
	pub const fn size(self) -> VertexAttributeSize {
		match self as u8 % 4 {
			0 => VertexAttributeSize::One,
			1 => VertexAttributeSize::Two,
			2 => VertexAttributeSize::Three,
			3 => VertexAttributeSize::Four,
			_ => unreachable!(),
		}
	}
	/// Returns the type of the vertex attribute.
	#[inline]
	pub const fn ty(self) -> VertexAttributeType {
		match self as u8 / 4 {
			0 => VertexAttributeType::F32,
			1 => VertexAttributeType::F64,
			2 => VertexAttributeType::I32,
			3 => VertexAttributeType::U32,
			4 => VertexAttributeType::I16,
			5 => VertexAttributeType::U16,
			6 => VertexAttributeType::I8,
			7 => VertexAttributeType::U8,
			// Normalized types
			8 => VertexAttributeType::I16,
			9 => VertexAttributeType::U16,
			10 => VertexAttributeType::I8,
			11 => VertexAttributeType::U8,
			_ => unreachable!(),
		}
	}
	/// Returns `true` if the vertex attribute format is normalized.
	#[inline]
	pub const fn normalized(self) -> bool {
		self as usize >= VertexAttributeFormat::I16Norm as usize
	}
}

/// Normalized attribute.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Norm<T>(pub T);
unsafe impl<T: dataview::Pod> dataview::Pod for Norm<T> {}

#[macro_export]
macro_rules! norm {
	([$($e:expr),* $(,)?]) => {
		[$($crate::Norm($e)),*]
	}
}

pub trait TVertexAttributeFormat {
	const FORMAT: VertexAttributeFormat;
}

macro_rules! TVertexAttributeFormat {
	($ty:ty; $v:ident, $v2:ident, $v3:ident, $v4:ident) => {
		impl TVertexAttributeFormat for $ty {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v;
		}
		impl TVertexAttributeFormat for [$ty; 1] {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v;
		}
		impl TVertexAttributeFormat for [$ty; 2] {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v2;
		}
		impl TVertexAttributeFormat for [$ty; 3] {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v3;
		}
		impl TVertexAttributeFormat for [$ty; 4] {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v4;
		}
		impl TVertexAttributeFormat for cvmath::Vec2<$ty> {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v2;
		}
		impl TVertexAttributeFormat for cvmath::Vec3<$ty> {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v3;
		}
		impl TVertexAttributeFormat for cvmath::Vec4<$ty> {
			const FORMAT: VertexAttributeFormat = VertexAttributeFormat::$v4;
		}
	}
}

TVertexAttributeFormat!(f32; F32, F32v2, F32v3, F32v4);
TVertexAttributeFormat!(f64; F64, F64v2, F64v3, F64v4);
TVertexAttributeFormat!(i32; I32, I32v2, I32v3, I32v4);
TVertexAttributeFormat!(u32; U32, U32v2, U32v3, U32v4);
TVertexAttributeFormat!(i16; I16, I16v2, I16v3, I16v4);
TVertexAttributeFormat!(u16; U16, U16v2, U16v3, U16v4);
TVertexAttributeFormat!(i8; I8, I8v2, I8v3, I8v4);
TVertexAttributeFormat!(u8; U8, U8v2, U8v3, U8v4);
TVertexAttributeFormat!(Norm<i16>; I16Norm, I16Normv2, I16Normv3, I16Normv4);
TVertexAttributeFormat!(Norm<u16>; U16Norm, U16Normv2, U16Normv3, U16Normv4);
TVertexAttributeFormat!(Norm<i8>; I8Norm, I8Normv2, I8Normv3, I8Normv4);
TVertexAttributeFormat!(Norm<u8>; U8Norm, U8Normv2, U8Normv3, U8Normv4);

/// Vertex attribute.
#[derive(Copy, Clone, Debug)]
pub struct VertexAttribute {
	/// Name of the attribute variable in the shader.
	pub name: &'static str,
	/// Format of the vertex attribute.
	pub format: VertexAttributeFormat,
	/// Offset of the attribute in the vertex structure.
	pub offset: u16,
}

impl VertexAttribute {
	pub const fn with<T: TVertexAttributeFormat>(name: &'static str, offset: usize) -> VertexAttribute {
		VertexAttribute {
			name,
			format: T::FORMAT,
			offset: offset as u16,
		}
	}
}

/// Vertex layout.
#[derive(Copy, Clone, Debug)]
pub struct VertexLayout {
	pub size: u16,
	pub alignment: u16,
	pub attributes: &'static [VertexAttribute],
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
/// Vertex divisor for instanced rendering.
pub enum VertexDivisor {
	/// The attribute advances per vertex (default behavior).
	#[default]
	PerVertex,
	/// The attribute advances per instance.
	PerInstance,
	// /// The attribute advances once every `n` instances.
	// Divisor(u32),
}
