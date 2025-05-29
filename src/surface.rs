define_handle!(Surface);

impl Surface {
	/// Back buffer surface.
	pub const BACK_BUFFER: Surface = Surface(0);
}

/// Surface format.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum SurfaceFormat {
	RGBA8,
	RGB8,
}

/// Surface information.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct SurfaceInfo {
	pub offscreen: bool,
	pub has_depth: bool,
	pub has_texture: bool,
	pub format: SurfaceFormat,
	pub width: i32,
	pub height: i32,
}
