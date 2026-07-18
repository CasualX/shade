use super::*;

/// Vertex buffer resource.
pub trait VertexBuffer: Resource {
	fn layout(&self) -> &'static crate::vertex::VertexLayout;
}

impl Eq for dyn VertexBuffer + '_ {}

impl PartialEq for dyn VertexBuffer + '_ {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		ptr::addr_eq(self, other)
	}
}

impl fmt::Debug for dyn VertexBuffer + '_ {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.fmt(f)
	}
}

/// Index buffer resource.
pub trait IndexBuffer: Resource {
	fn index_type(&self) -> IndexType;
}

impl Eq for dyn IndexBuffer + '_ {}

impl PartialEq for dyn IndexBuffer + '_ {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		ptr::addr_eq(self, other)
	}
}

impl fmt::Debug for dyn IndexBuffer + '_ {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.fmt(f)
	}
}

/// Index type for index buffers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IndexType { U16, U32 }

impl IndexType {
	#[inline]
	pub const fn size(self) -> usize {
		match self {
			IndexType::U16 => 2,
			IndexType::U32 => 4,
		}
	}
}

/// Trait for index types.
pub trait TIndex: Copy + Ord + Default + dataview::Pod + fmt::Debug {
	const TYPE: IndexType;
}

impl TIndex for u16 {
	const TYPE: IndexType = IndexType::U16;
}
impl TIndex for u32 {
	const TYPE: IndexType = IndexType::U32;
}

/// Trait for index collections.
pub trait TIndices {
	type Index: TIndex;

	fn as_indices(&self) -> &[Self::Index];
}
impl<T: TIndex> TIndices for [T] {
	type Index = T;

	#[inline]
	fn as_indices(&self) -> &[Self::Index] {
		self
	}
}
impl<T: TIndex, const N: usize> TIndices for [T; N] {
	type Index = T;

	#[inline]
	fn as_indices(&self) -> &[Self::Index] {
		self.as_slice()
	}
}
impl<T: TIndex> TIndices for Vec<T> {
	type Index = T;

	#[inline]
	fn as_indices(&self) -> &[Self::Index] {
		self.as_slice()
	}
}

/// Triangle indices.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Index3 {
	pub p1: u32,
	pub p2: u32,
	pub p3: u32,
}
unsafe impl dataview::Pod for Index3 {}
impl TIndices for Vec<Index3> {
	type Index = u32;

	#[inline]
	fn as_indices(&self) -> &[Self::Index] {
		unsafe { slice::from_raw_parts(self.as_ptr() as *const u32, self.len() * 3) }
	}
}

/// Primitive type.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PrimType {
	/// Triangles.
	Triangles,
	/// Lines.
	Lines,
}

/// Blend mode.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum BlendMode {
	/// Solid color.
	///
	/// ```text
	/// result[rgb] = src[rgb]
	/// result[a] = src[a]
	/// ```
	#[default]
	Solid,

	/// Alpha blending.
	///
	/// ```text
	/// result[rgb] = src[rgb] * src[a] + dest[rgb] * (1 - src[a])
	/// result[a] = src[a] + dest[a] * (1 - src[a])
	/// ```
	Alpha,

	/// Premultiplied alpha blending.
	///
	/// ```text
	/// result[rgb] = src[rgb] + dest[rgb] * (1 - src[a])
	/// result[a] = src[a] + dest[a] * (1 - src[a])
	/// ```
	PremultipliedAlpha,

	/// Additive blending.
	///
	/// ```text
	/// result[rgb] = src[rgb] + dest[rgb]
	/// ```
	Additive,

	/// Maximum of the src and dest colors.
	///
	/// ```text
	/// result[rgb] = max(src[rgb], dest[rgb])
	/// ```
	Lighten,

	/// Screen effect.
	///
	/// ```text
	/// result[rgb] = src[rgb] + (1 - dest[rgb])
	/// ```
	Screen,

	/// Minimum of the src and dest colors.
	///
	/// ```text
	/// result[rgb] = min(src[rgb], dest[rgb])
	/// ```
	Darken,

	/// Multiply blending.
	///
	/// ```text
	/// result[rgb] = src[rgb] * dest[rgb]
	/// ```
	Multiply,
}

/// Comparison operator.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Compare {
	/// Never pass.
	Never,
	/// Pass if the new value is less than the old value.
	Less,
	/// Pass if the new value is equal to the old value.
	Equal,
	/// Pass if the new value is not equal to the old value.
	NotEqual,
	/// Pass if the new value is less than or equal to the old value.
	LessEqual,
	/// Pass if the new value is greater than the old value.
	Greater,
	/// Pass if the new value is greater than or equal to the old value.
	GreaterEqual,
	/// Always pass.
	Always,
}

/// Cull mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CullMode {
	/// Cull counter-clockwise faces.
	CCW,
	/// Cull clockwise faces.
	CW,
}

/// Buffer usage.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BufferUsage {
	/// Buffer is static and used frequently.
	Static,
	/// Buffer is dynamic and used frequently.
	Dynamic,
	/// Buffer is streamed and used infrequently.
	Stream,
}
