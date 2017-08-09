
/// Primitive types.
// DO NOT CHANGE THESE VALUES!
// The value represents the number of indices per primitive.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Primitive {
	#[doc(hidden)]
	Unspecified = 0,
	/// List of points.
	Points = 1,
	/// List of line segments.
	Lines = 2,
	/// List of triangles.
	Triangles = 3,
}
impl Default for Primitive {
	fn default() -> Primitive {
		Primitive::Points
	}
}

/// Blend modes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Blend {
	/// No alpha blending.
	Solid,
	/// Standard alpha blending.
	///
	/// ```c
	/// result_c = src_c * src_a + dest_c * (1 - src_a);
	/// result_a = src_a + src_c;
	/// ```
	Alpha,
	/// Blend with `max` function.
	Lighten,
	Screen,
	/// Blend with `min` function.
	Darken,
	/// Multiply color components.
	///
	/// ```c
	/// result_c = src_c * dest_c
	/// result_a = src_a * dest_a
	/// ```
	Multiply,
	/// Additive blending.
	///
	/// ```c
	/// result_c = src_c + dest_c
	/// result_a = src_a + dest_a
	/// ```
	Additive,
	Invert,
}
impl Default for Blend {
	fn default() -> Blend {
		Blend::Solid
	}
}

/// Debug visualizations.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Visualize {
	/// Normal rendering.
	Normal,
	/// Visualize overdraw.
	Overdraw,
	/// Visualize batching.
	Batching,
	/// Visualize wireframes.
	Wireframe,
}
impl Default for Visualize {
	fn default() -> Visualize {
		Visualize::Normal
	}
}

/// Stencil buffer settings.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Stencil {
	/// Draw to stencil buffer.
	Clip(u8),
	/// Draw pixels that have stencil value.
	Inside(u8),
	/// Draw pixels that does not have stencil value.
	Outside(u8),
}
