define_handle!(VertexBuffer);
define_handle!(IndexBuffer);

/// Index type for index buffers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IndexType { U8, U16, U32 }

impl IndexType {
	pub const fn size(self) -> usize {
		match self {
			IndexType::U8 => 1,
			IndexType::U16 => 2,
			IndexType::U32 => 4,
		}
	}
}

/// Trait for index types.
pub trait TIndex: Copy + dataview::Pod {
	const TYPE: IndexType;
}

impl TIndex for u8 {
	const TYPE: IndexType = IndexType::U8;
}
impl TIndex for u16 {
	const TYPE: IndexType = IndexType::U16;
}
impl TIndex for u32 {
	const TYPE: IndexType = IndexType::U32;
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

/// Depth test.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DepthTest {
	/// Never pass.
	Never,
	/// Pass if the new depth is less than the old depth.
	Less,
	/// Pass if the new depth is equal to the old depth.
	Equal,
	/// Pass if the new depth is not equal to the old depth.
	NotEqual,
	/// Pass if the new depth is less than or equal to the old depth.
	LessEqual,
	/// Pass if the new depth is greater than the old depth.
	Greater,
	/// Pass if the new depth is greater than or equal to the old depth.
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
