use std::collections::HashMap;
use std::{error, fmt, hash, str};
use cvmath::*;

#[cfg(feature = "serde")]
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	*value == T::default()
}

#[cfg(feature = "serde")]
mod ordered_hashmap {
	use std::collections::HashMap;
	use std::hash::Hash;
	use serde::ser::SerializeMap;

	pub fn serialize<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
	where
		K: Hash + Ord + serde::Serialize,
		V: serde::Serialize,
		S: serde::Serializer,
	{
		let mut keys = map.keys().collect::<Vec<_>>();
		keys.sort_unstable();

		let mut ordered = serializer.serialize_map(Some(keys.len()))?;
		for key in keys {
			ordered.serialize_entry(key, &map[key])?;
		}
		ordered.end()
	}

	pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
	where
		K: Eq + Hash + serde::Deserialize<'de>,
		V: serde::Deserialize<'de>,
		D: serde::Deserializer<'de>,
	{
		<HashMap<K, V> as serde::Deserialize>::deserialize(deserializer)
	}
}

#[cfg(feature = "serde")]
mod rect_array {
	pub fn serialize<S>(rect: &super::Recti, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		<[i32; 4] as serde::Serialize>::serialize(&[rect.x, rect.y, rect.width, rect.height], serializer)
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<super::Recti, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let [x, y, width, height] = <[i32; 4] as serde::Deserialize>::deserialize(deserializer)?;
		Ok(super::Recti { x, y, width, height })
	}
}

#[cfg(feature = "serde")]
mod ordered_kerning {
	use std::collections::HashMap;

	#[derive(serde::Serialize, serde::Deserialize)]
	struct KerningPair {
		first: u32,
		second: u32,
		#[serde(default, skip_serializing_if = "super::is_default")]
		metrics_index: u32,
		advance: f32,
	}

	pub fn serialize<S>(map: &HashMap<(u32, u32), super::Kerning>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut keys = map.keys().collect::<Vec<_>>();
		keys.sort_unstable();

		let pairs = keys.into_iter().map(|key| {
			let &super::Kerning { metrics_index, advance } = &map[key];
			let &(first, second) = key;
			KerningPair { first, second, metrics_index, advance }
		});
		serializer.collect_seq(pairs)
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<(u32, u32), super::Kerning>, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let pairs = <Vec<KerningPair> as serde::Deserialize>::deserialize(deserializer)?;
		fn map_kerning_pair(KerningPair { first, second, metrics_index, advance }: KerningPair) -> ((u32, u32), super::Kerning) {
			((first, second), super::Kerning { metrics_index, advance })
		}
		Ok(pairs.into_iter().map(map_kerning_pair).collect())
	}
}

/// A single sprite frame in the atlas.
#[derive(Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
	/// The tight drawable bounds of the sprite in the atlas image.
	#[cfg_attr(feature = "serde", serde(with = "rect_array"))]
	pub rect: Recti,
	/// Extra atlas pixels around [`Frame::rect`] used by editing and repacking tools.
	///
	/// The margin is outside the drawable rect and is not sampled during normal sprite drawing.
	/// The full occupied atlas area is `rect` inflated by this many pixels on each side.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub margin: i32,
	/// UV transform to apply when sampling this frame.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub transform: Transform,
	/// The origin of the sprite in pixels, relative to the top-left corner of [`Frame::rect`].
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub origin: Vec2<i32>,
}

impl Frame {
	/// Returns the sprite quad in atlas pixel units, transformed by [`Frame::transform`].
	pub fn get_sprite(&self) -> crate::d2::Sprite<Vec2i> {
		crate::d2::Sprite {
			bottom_left: Vec2(self.rect.left(), self.rect.bottom()),
			top_left: Vec2(self.rect.left(), self.rect.top()),
			top_right: Vec2(self.rect.right(), self.rect.top()),
			bottom_right: Vec2(self.rect.right(), self.rect.bottom()),
		}.transform(self.transform)
	}
}

/// A frame in an animated sprite.
#[derive(Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedFrame {
	/// The frame rectangle and sampling metadata.
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub frame: Frame,
	/// Frame duration in seconds.
	pub duration: f32,
}

/// Named sprite data in the atlas.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Sprite {
	/// A single-frame sprite.
	Frame(Frame),
	/// An animated sprite with per-frame durations.
	Animated(Vec<AnimatedFrame>),
}

impl Sprite {
	#[inline]
	pub fn as_frame(&self) -> Option<&Frame> {
		match self {
			Sprite::Frame(frame) => Some(frame),
			Sprite::Animated(_) => None,
		}
	}
	#[inline]
	pub fn as_animated(&self) -> Option<&[AnimatedFrame]> {
		match self {
			Sprite::Frame(_) => None,
			Sprite::Animated(frames) => Some(frames),
		}
	}
}

impl Sprite {
	/// Returns the number of frames in this sprite.
	#[inline]
	pub fn len(&self) -> usize {
		match self {
			Sprite::Frame(_) => 1,
			Sprite::Animated(frames) => frames.len(),
		}
	}
	/// Returns `true` if this sprite has no frames.
	#[inline]
	pub fn is_empty(&self) -> bool {
		match self {
			Sprite::Frame(_) => false,
			Sprite::Animated(frames) => frames.is_empty(),
		}
	}
	/// Returns the frame at the given index, or `None` if the index is out of bounds.
	#[inline]
	pub fn get_frame(&self, index: usize) -> Option<&Frame> {
		match self {
			Sprite::Frame(frame) if index == 0 => Some(frame),
			Sprite::Frame(_) => None,
			Sprite::Animated(frames) => frames.get(index).map(|f| &f.frame),
		}
	}
	/// Returns the frame at the given index, wrapping around if the index is out of bounds.
	///
	/// If the sprite has no frames, returns `None`.
	#[inline]
	pub fn get_frame_wrapping(&self, index: usize) -> Option<&Frame> {
		match self {
			Sprite::Frame(frame) => Some(frame),
			Sprite::Animated(frames) if frames.is_empty() => None,
			Sprite::Animated(frames) => Some(&frames[index % frames.len()].frame),
		}
	}
	/// Returns the total duration of this sprite in seconds.
	///
	/// Single-frame sprites have a duration of `1.0`.
	#[inline]
	pub fn duration(&self) -> f32 {
		match self {
			Sprite::Frame(_) => 1.0,
			Sprite::Animated(frames) => frames.iter().map(|f| f.duration).sum(),
		}
	}
	/// Returns the frame at the given time in seconds, clamping to the first and last frames if the time is out of bounds.
	///
	/// The `time` parameter is relative to the start of the animation. Callers that want looping
	/// playback should wrap `time` before calling this method, using the duration of an animated
	/// sprite with a positive total duration.
	///
	/// Animated frame durations are expected to be positive, and animated sprites are expected to
	/// have a positive total duration. Non-positive durations and non-finite times are not
	/// specially handled and may produce surprising results.
	///
	/// If the sprite has no frames, returns `None`.
	pub fn get_frame_at(&self, mut time: f32) -> Option<&Frame> {
		match self {
			Sprite::Frame(frame) => Some(frame),
			Sprite::Animated(frames) if frames.is_empty() => None,
			Sprite::Animated(frames) => {
				for frame in frames {
					if time < frame.duration {
						return Some(&frame.frame);
					}
					time -= frame.duration;
				}
				Some(&frames.last().unwrap().frame)
			}
		}
	}
}

/// A discrete UV transform for a frame inside a texture atlas.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Transform {
	/// No transformation.
	#[default]
	None = 0,
	/// Rotate 90 degrees clockwise.
	Rotate90 = 1,
	/// Rotate 180 degrees.
	Rotate180 = 2,
	/// Rotate 270 degrees clockwise (or 90 degrees ccw).
	Rotate270 = 3,
	/// Flip horizontally.
	FlipX = 4,
	/// Flip across the `/` diagonal.
	FlipSlash = 5,
	/// Flip vertically.
	FlipY = 6,
	/// Flip across the `\` diagonal.
	FlipBackslash = 7,
}
impl Transform {
	/// Returns the composition of `self` followed by `rhs`.
	pub const fn then(self, rhs: Transform) -> Transform {
		let (self_rotation, self_reflect) = self.decompose();
		let (rhs_rotation, rhs_reflect) = rhs.decompose();

		let rotation = if rhs_reflect {
			rhs_rotation.wrapping_sub(self_rotation) & 3
		}
		else {
			rhs_rotation.wrapping_add(self_rotation) & 3
		};
		Transform::from_parts(rotation, self_reflect ^ rhs_reflect)
	}

	const fn decompose(self) -> (u8, bool) {
		match self {
			Transform::None => (0, false),
			Transform::Rotate90 => (1, false),
			Transform::Rotate180 => (2, false),
			Transform::Rotate270 => (3, false),
			Transform::FlipX => (0, true),
			Transform::FlipSlash => (1, true),
			Transform::FlipY => (2, true),
			Transform::FlipBackslash => (3, true),
		}
	}

	const fn from_parts(rotation: u8, reflect: bool) -> Transform {
		match (rotation & 3, reflect) {
			(0, false) => Transform::None,
			(1, false) => Transform::Rotate90,
			(2, false) => Transform::Rotate180,
			(3, false) => Transform::Rotate270,
			(0, true) => Transform::FlipX,
			(1, true) => Transform::FlipSlash,
			(2, true) => Transform::FlipY,
			(3, true) => Transform::FlipBackslash,
			_ => unreachable!(),
		}
	}
}

impl fmt::Display for Transform {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			Transform::None => "None",
			Transform::Rotate90 => "Rotate90",
			Transform::Rotate180 => "Rotate180",
			Transform::Rotate270 => "Rotate270",
			Transform::FlipX => "FlipX",
			Transform::FlipSlash => "FlipSlash",
			Transform::FlipY => "FlipY",
			Transform::FlipBackslash => "FlipBackslash",
		};
		f.write_str(s)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseTransformError {
	value: (),
}

impl fmt::Display for ParseTransformError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("unknown transform")
	}
}

impl error::Error for ParseTransformError {}

impl str::FromStr for Transform {
	type Err = ParseTransformError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let transform = match s {
			"None" | "none" => Transform::None,
			"Rotate90" | "rotate90" => Transform::Rotate90,
			"Rotate180" | "rotate180" => Transform::Rotate180,
			"Rotate270" | "rotate270" => Transform::Rotate270,
			"FlipX" | "flipx" => Transform::FlipX,
			"FlipSlash" | "flipslash" => Transform::FlipSlash,
			"FlipY" | "flipy" => Transform::FlipY,
			"FlipBackslash" | "flipbackslash" => Transform::FlipBackslash,
			_ => return Err(ParseTransformError { value: () }),
		};
		Ok(transform)
	}
}

/// Encoding used by the atlas texture.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Kind {
	/// Regular bitmap alpha or color.
	Bitmap,
	/// Monochrome (true) signed distance field.
	Sdf,
	/// Monochrome signed perpendicular distance field.
	Psdf,
	/// Multi-channel signed distance field.
	Msdf,
	/// Combined multi-channel and true signed distance field in the alpha channel.
	Mtsdf,
}

/// Texture metadata for sprite and font atlases.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata {
	/// The width of the atlas in pixels.
	pub width: i32,
	/// The height of the atlas in pixels.
	pub height: i32,
	/// The distance-field or bitmap encoding.
	pub kind: Kind,
	/// Width of the representable signed-distance range, in atlas pixels.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub distance_range: f32,
	/// Center point of the representable signed-distance range, in atlas pixels.
	///
	/// This is `0.0` for symmetric ranges such as `-2.0..2.0`, and non-zero for asymmetric ranges such as `-1.0..7.0`.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub distance_range_middle: f32,
}

/// Texture atlas containing named sprites and fonts.
///
/// Atlas coordinates use image-space axes: `[0, 0]` is the top-left pixel of the texture and `y` increases downward.
///
/// The generic parameters `K` and `F` are the key types for the [`Atlas::sprites`] and [`Atlas::fonts`] maps, respectively.
/// The default is `String` for both but can be replaced with an enum for compile-time key checking.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(
	serialize = "K: Ord + serde::Serialize, F: Ord + serde::Serialize",
	deserialize = "K: serde::Deserialize<'de>, F: serde::Deserialize<'de>"
)))]
pub struct Atlas<K: Eq + hash::Hash = String, F: Eq + hash::Hash = String> {
	/// Atlas format version.
	///
	/// Version `0` is the default in-memory layout.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub version: u32,
	/// Texture metadata for the atlas.
	pub meta: Metadata,
	/// Named sprite animations in this atlas.
	#[cfg_attr(feature = "serde", serde(with = "ordered_hashmap"))]
	pub sprites: HashMap<K, Sprite>,
	/// Font metadata embedded in the same atlas, keyed by font name.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "HashMap::is_empty", with = "ordered_hashmap"))]
	pub fonts: HashMap<F, Font>,
}

/// Bounds of a glyph quad in font plane coordinates.
///
/// These are font-space coordinates (`left, top, right, bottom`). The y-axis increases downward.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlaneBounds {
	pub left: f32,
	pub top: f32,
	pub right: f32,
	pub bottom: f32,
}

/// Plane and Glyph bounds for a single glyph in the atlas.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlyphBounds {
	/// The glyph quad in font-plane coordinates.
	pub plane_bounds: PlaneBounds,
	/// The bounds of the glyph in the atlas image.
	#[cfg_attr(feature = "serde", serde(with = "rect_array"))]
	pub atlas_bounds: Recti,
}

/// Font-wide metrics for a set of glyphs that share the same em size.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Metrics {
	/// Units per em for this metrics set.
	///
	/// Glyph plane coordinates and advances are divided by this value.
	pub em_size: f32,
	/// Recommended distance between consecutive baselines.
	pub line_height: f32,
	/// Distance from the baseline to the top of typical ascenders.
	pub ascender: f32,
	/// Distance from the baseline to the bottom of typical descenders.
	pub descender: f32,
	/// Baseline-relative y position of the underline.
	pub underline_y: f32,
	/// Recommended underline stroke thickness.
	pub underline_thickness: f32,
}

/// A glyph inside the atlas.
///
/// Glyphs with missing bounds still contribute advance when laid out, but do
/// not emit a quad. This is useful for spaces and other invisible glyphs.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Glyph {
	/// Fonts may contain glyphs of multiple fonts, identified by their metrics index.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub metrics_index: u32,
	/// Horizontal cursor advance in font units.
	pub advance: f32,
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub bounds: Option<GlyphBounds>,
}

/// Horizontal adjustment between two glyphs.
///
/// Kerning is keyed by `(previous_glyph, current_glyph)` in [`Font::kerning`]
/// and is applied before positioning the current glyph.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Kerning {
	/// Metrics set used to scale this kerning adjustment.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_default"))]
	pub metrics_index: u32,
	/// Horizontal cursor adjustment in font units.
	pub advance: f32,
}

/// Runtime font metadata for atlas glyphs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Font {
	/// Texture metadata for the font atlas.
	pub meta: Metadata,
	/// Metrics sets referenced by glyphs.
	///
	/// Multiple sets allow one atlas to contain variants with different em sizes or font faces.
	/// [`Glyph::metrics_index`] selects the entry used for scaling and cursor advance.
	pub metrics: Vec<Metrics>,
	/// Glyph table indexed by glyph ID.
	pub glyphs: Vec<Glyph>,
	/// Character-to-glyph lookup for basic text layout.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "HashMap::is_empty", with = "ordered_hashmap"))]
	pub codepoints: HashMap<char, u32>,
	/// Kerning adjustments keyed by `(previous_glyph, current_glyph)`.
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "HashMap::is_empty", with = "ordered_kerning"))]
	pub kerning: HashMap<(u32, u32), Kerning>,
}

impl crate::d2::IFont for Font {
	fn write_span(&self, mut cv: Option<&mut (dyn crate::d2::ITextTarget + '_)>, scribe: &crate::d2::Scribe, cursor: &mut crate::d2::Cursor, text: &str) {
		for chr in text.chars() {
			let Some(&glyph_index) = self.codepoints.get(&chr) else { continue };
			let Some(glyph) = self.glyphs.get(glyph_index as usize) else { continue };
			if let Some(kerning) = self.kerning.get(&(cursor.previous_glyph, glyph_index)) {
				let metrics = &self.metrics[kerning.metrics_index as usize];
				let scale = scribe.font_size / metrics.em_size;
				let scale_x = scribe.font_width_scale * scale;
				cursor.pos.x += kerning.advance * scale_x;
			}

			let metrics = &self.metrics[glyph.metrics_index as usize];
			let pos = cursor.pos + Vec2(0.0, scribe.line_height - scribe.font_size - scribe.baseline);

			let scale = scribe.font_size / metrics.em_size;
			let scale_x = scribe.font_width_scale * scale;
			let scale_y = scale;
			let advance = glyph.advance * scale_x + scribe.letter_spacing;
			cursor.pos.x += advance;
			cursor.previous_glyph = glyph_index;

			if let Some(cv) = cv.as_deref_mut() {
				let Some(bounds) = &glyph.bounds else { continue };

				let plane_bounds = Bounds2!(
					bounds.plane_bounds.left * scale_x,
					bounds.plane_bounds.top * scale_y,
					bounds.plane_bounds.right * scale_x,
					bounds.plane_bounds.bottom * scale_y,
				);
				let atlas_bounds = bounds.atlas_bounds;
				let atlas_width = self.meta.width as f32;
				let atlas_height = self.meta.height as f32;
				let aleft = (atlas_bounds.left() as f32 + 0.5) / atlas_width;
				let aright = (atlas_bounds.right() as f32 - 0.5) / atlas_width;
				let atop = (atlas_bounds.top() as f32 + 0.5) / atlas_height;
				let abottom = (atlas_bounds.bottom() as f32 - 0.5) / atlas_height;

				let vertices = crate::d2::Sprite {
					bottom_left: crate::d2::TextVertex {
						pos: pos + plane_bounds.bottom_left(),
						uv: Vec2(aleft, abottom),
						color: scribe.color,
						outline: scribe.outline,
						data: crate::d2::TextVertexData::BOTTOM_LEFT,
					},
					top_left: crate::d2::TextVertex {
						pos: pos + plane_bounds.top_left() + Vec2(scribe.top_skew, 0.0),
						uv: Vec2(aleft, atop),
						color: scribe.color,
						outline: scribe.outline,
						data: crate::d2::TextVertexData::TOP_LEFT,
					},
					top_right: crate::d2::TextVertex {
						pos: pos + plane_bounds.top_right() + Vec2(scribe.top_skew, 0.0),
						uv: Vec2(aright, atop),
						color: scribe.color,
						outline: scribe.outline,
						data: crate::d2::TextVertexData::TOP_RIGHT,
					},
					bottom_right: crate::d2::TextVertex {
						pos: pos + plane_bounds.bottom_right(),
						uv: Vec2(aright, abottom),
						color: scribe.color,
						outline: scribe.outline,
						data: crate::d2::TextVertexData::BOTTOM_RIGHT,
					},
				};

				cv.text_quad(&vertices);
			}
		}
	}
}
