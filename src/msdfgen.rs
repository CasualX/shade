/*!
 * Use fonts created with Msdfgen with libshade.
 *
 * * <https://github.com/Chlumsky/msdfgen>
 * * <https://github.com/Chlumsky/msdf-atlas-gen>
 * * <https://www.redblobgames.com/x/2404-distance-field-effects/>
 */

use std::collections::HashMap;
use std::{error, fmt, str};
use crate::atlas;

impl From<FontDto> for atlas::Font {
	fn from(dto: FontDto) -> Self {
		// Collect metrics and glyphs from either a single or multiple variants
		let mut metrics: Vec<atlas::Metrics> = Vec::new();
		let mut glyphs: Vec<atlas::Glyph> = Vec::new();
		let mut codepoints: HashMap<char, u32> = HashMap::new();
		let mut kerning: HashMap<(u32, u32), atlas::Kerning> = HashMap::new();
		let atlas = dto.atlas;
		let atlas_height = atlas.height as f32;
		let y_origin = atlas.y_origin;

		let variants = match dto.variants {
			FontVariants::Single(variant) => vec![variant],
			FontVariants::Multiple { variants } => variants,
		};

		for (metrics_index, variant) in variants.into_iter().enumerate() {
			let metrics_index = metrics_index as u32;
			metrics.push(variant.metrics);
			let mut variant_codepoints: HashMap<char, u32> = HashMap::new();
			for g in variant.glyphs.into_iter() {
				let glyph_index = glyphs.len() as u32;
				let bounds = match (g.plane_bounds, g.atlas_bounds) {
					(Some(plane_bounds), Some(atlas_bounds)) => Some(atlas::GlyphBounds {
						plane_bounds: plane_bounds.into(),
						atlas_bounds: y_origin.normalize_atlas_bounds(atlas_bounds, atlas_height),
					}),
					_ => None,
				};
				glyphs.push(atlas::Glyph {
					metrics_index,
					advance: g.advance,
					bounds,
				});
				if let Some(chr) = char::from_u32(g.unicode) {
					// If multiple variants contain the same codepoint, last one wins.
					codepoints.insert(chr, glyph_index);
					variant_codepoints.insert(chr, glyph_index);
				}
			}
			for KerningDto { first, second, advance } in variant.kerning.into_iter() {
				let Some(first) = char::from_u32(first) else { continue };
				let Some(second) = char::from_u32(second) else { continue };
				let Some(&first_glyph) = variant_codepoints.get(&first) else { continue };
				let Some(&second_glyph) = variant_codepoints.get(&second) else { continue };
				kerning.insert((first_glyph, second_glyph), atlas::Kerning { metrics_index, advance });
			}
		}

		let meta = atlas::Metadata {
			width: atlas.width,
			height: atlas.height,
			kind: atlas.mode.into(),
			distance_range: atlas.distance_range,
			distance_range_middle: atlas.distance_range_middle,
		};

		atlas::Font {
			meta,
			metrics,
			glyphs,
			codepoints,
			kerning,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FontDto {
	pub atlas: Atlas,
	#[serde(flatten)]
	pub variants: FontVariants,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum FontVariants {
	Single(FontVariant),
	Multiple {
		variants: Vec<FontVariant>,
	},
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontVariant {
	pub metrics: atlas::Metrics,
	pub glyphs: Vec<GlyphDto>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub kerning: Vec<KerningDto>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Atlas {
	#[serde(rename = "type")]
	pub mode: Mode,
	pub distance_range: f32,
	pub distance_range_middle: f32,
	pub size: f32,
	pub width: i32,
	pub height: i32,
	pub y_origin: YOrigin,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
	/// Monochrome (true) signed distance field.
	Sdf,
	/// Monochrome signed perpendicular distance field.
	Psdf,
	/// Multi-channel signed distance field.
	Msdf,
	/// Combined multi-channel and true signed distance field in the alpha channel.
	Mtsdf,
}

impl Mode {
	#[inline]
	pub fn as_str(self) -> &'static str {
		match self {
			Mode::Sdf => "sdf",
			Mode::Psdf => "psdf",
			Mode::Msdf => "msdf",
			Mode::Mtsdf => "mtsdf",
		}
	}
}

impl fmt::Display for Mode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ParseModeError;

impl fmt::Display for ParseModeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("unknown msdfgen mode")
	}
}

impl error::Error for ParseModeError {}

impl str::FromStr for Mode {
	type Err = ParseModeError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"sdf" => Ok(Mode::Sdf),
			"psdf" => Ok(Mode::Psdf),
			"msdf" => Ok(Mode::Msdf),
			"mtsdf" => Ok(Mode::Mtsdf),
			_ => Err(ParseModeError),
		}
	}
}

impl From<Mode> for atlas::Kind {
	fn from(value: Mode) -> Self {
		match value {
			Mode::Sdf => atlas::Kind::Sdf,
			Mode::Psdf => atlas::Kind::Psdf,
			Mode::Msdf => atlas::Kind::Msdf,
			Mode::Mtsdf => atlas::Kind::Mtsdf,
		}
	}
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum FillRule {
	#[default]
	NonZero,
	EvenOdd,
	Positive,
	Negative,
}

impl FillRule {
	#[inline]
	pub fn as_str(self) -> &'static str {
		match self {
			FillRule::NonZero => "nonzero",
			FillRule::EvenOdd => "evenodd",
			FillRule::Positive => "positive",
			FillRule::Negative => "negative",
		}
	}
}

impl str::FromStr for FillRule {
	type Err = ();

	fn from_str(value: &str) -> Result<FillRule, Self::Err> {
		match value {
			"nonzero" => Ok(FillRule::NonZero),
			"evenodd" => Ok(FillRule::EvenOdd),
			"positive" => Ok(FillRule::Positive),
			"negative" => Ok(FillRule::Negative),
			_ => Err(()),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum YOrigin {
	Bottom,
	Top,
}

impl Default for YOrigin {
	fn default() -> Self {
		YOrigin::Bottom
	}
}

impl YOrigin {
	fn normalize_atlas_bounds(&self, bounds: Bounds, atlas_height: f32) -> cvmath::Recti {
		let (left, top, right, bottom) = match self {
			YOrigin::Bottom => (bounds.left, atlas_height - bounds.top, bounds.right, atlas_height - bounds.bottom),
			YOrigin::Top => (bounds.left, bounds.top, bounds.right, bounds.bottom),
		};
		cvmath::Recti {
			x: left.floor() as i32,
			y: top.floor() as i32,
			width: (right.ceil() - left.floor()) as i32,
			height: (bottom.ceil() - top.floor()) as i32,
		}
	}
}

/// Axis-aligned bounds as emitted by `msdf-atlas-gen`.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bounds {
	/// Minimum x coordinate.
	pub left: f32,
	/// Minimum y coordinate.
	pub bottom: f32,
	/// Maximum x coordinate.
	pub right: f32,
	/// Maximum y coordinate.
	pub top: f32,
}

impl From<Bounds> for atlas::PlaneBounds {
	fn from(bounds: Bounds) -> Self {
		atlas::PlaneBounds {
			left: bounds.left,
			top: 1.0 - bounds.top,
			right: bounds.right,
			bottom: 1.0 - bounds.bottom,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphDto {
	pub unicode: u32,
	pub advance: f32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub plane_bounds: Option<Bounds>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub atlas_bounds: Option<Bounds>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KerningDto {
	#[serde(alias = "unicode1")]
	pub first: u32,
	#[serde(alias = "unicode2")]
	pub second: u32,
	pub advance: f32,
}
