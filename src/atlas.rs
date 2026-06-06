use std::collections::BTreeMap;
use cvmath::*;

fn default_len() -> u16 { 1 }
fn is_default_len(len: &u16) -> bool { *len == default_len() }

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	*value == T::default()
}

/// An animated sprite description.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Sprite {
	/// The index of the first frame.
	pub index: u16,
	/// The number of frames in the animation.
	#[serde(default = "default_len", skip_serializing_if = "is_default_len")]
	pub len: u16,
	/// The duration of the animation in seconds.
	#[serde(skip)] // Set to the sum of the durations of the frames when loaded.
	pub duration: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum Transform {
	/// No transformation.
	#[default]
	None,
	/// Flip horizontally.
	FlipX,
	/// Flip vertically.
	FlipY,
	/// Flip both horizontally and vertically.
	FlipXY,
	/// Rotate 90 degrees clockwise.
	Rotate90,
	/// Rotate 180 degrees.
	Rotate180,
	/// Rotate 270 degrees clockwise (or 90 degrees counter-clockwise).
	Rotate270,
}

/// A single frame in the sprite sheet.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Frame {
	/// The location of the sprite in the image
	///
	/// `[x, y, width, height]` in pixels.
	pub rect: [i32; 4],
	/// Sprite UV transform to apply when rendering.
	#[serde(default, skip_serializing_if = "is_default")]
	pub transform: Transform,
	/// The origin of the sprite in pixels, relative to the top-left corner of the rect.
	#[serde(default, skip_serializing_if = "is_default")]
	pub origin: Vec2<i32>,
	/// Frame duration in seconds.
	#[serde(default, skip_serializing_if = "is_default")]
	pub duration: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct SpriteSheet<T> {
	/// The width of the atlas in pixels.
	pub width: i32,
	/// The height of the atlas in pixels.
	pub height: i32,
	#[serde(bound(deserialize = "BTreeMap<T, Sprite>: serde::Deserialize<'de>"))]
	pub sprites: BTreeMap<T, Sprite>,
	pub frames: Vec<Frame>,
}
