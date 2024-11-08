// use super::*;

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
