
/// Defines a vertex type.
pub unsafe trait TVertex: Copy + Default + dataview::Pod {
	const VERTEX_LAYOUT: &'static VertexLayout;
}

/// Vertex attribute format.
#[derive(Copy, Clone, Debug)]
pub enum VertexAttributeFormat {
	/// 32-bit floating point number.
	F32,

	/// 64-bit floating point number.
	F64,

	/// 32-bit signed integer.
	I32,

	/// 32-bit unsigned integer.
	U32,

	/// 16-bit signed integer.
	I16,

	/// 16-bit unsigned integer.
	U16,

	/// 8-bit signed integer.
	I8,

	/// 8-bit unsigned integer.
	U8,

	/// Normalized `i16`.
	///
	/// The value is mapped from the range `[-32768, 32767]` to `[-1.0, 1.0]`.
	I16Norm,

	/// Normalized `u16`.
	///
	/// The value is mapped from the range `[0, 65535]` to `[0.0, 1.0]`.
	U16Norm,

	/// Normalized `i8`.
	///
	/// The value is mapped from the range `[-128, 127]` to `[-1.0, 1.0]`.
	I8Norm,

	/// Normalized `u8`.
	///
	/// The value is mapped from the range `[0, 255]` to `[0.0, 1.0]`.
	U8Norm,
}

/// Vertex attribute.
#[derive(Copy, Clone, Debug)]
pub struct VertexAttribute {
	pub format: VertexAttributeFormat,
	pub len: u16,
	pub offset: u16,
}

/// Vertex memory layout.
#[derive(Copy, Clone, Debug)]
pub struct VertexLayout {
	pub size: u16,
	pub alignment: u16,
	pub attributes: &'static [VertexAttribute],
}
