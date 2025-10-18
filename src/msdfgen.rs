/*!
 * Use fonts created with Msdfgen with libshade.
 *
 * * <https://github.com/Chlumsky/msdfgen>
 * * <https://github.com/Chlumsky/msdf-atlas-gen>
 * * <https://www.redblobgames.com/x/2404-distance-field-effects/>
 */

use std::collections::HashMap;

pub struct Font {
	pub atlas: Atlas,
	pub metrics: Vec<Metrics>,
	pub glyphs: HashMap<u32, Glyph>,
}

pub struct Glyph {
	pub metrics_index: usize,
	pub unicode: u32,
	pub advance: f32,
	pub plane_bounds: Option<Bounds>,
	pub atlas_bounds: Option<Bounds>,
}

impl From<FontDto> for Font {
	fn from(dto: FontDto) -> Self {
		// Collect metrics and glyphs from either a single or multiple variants
		let mut metrics: Vec<Metrics> = Vec::new();
		let mut glyphs: HashMap<u32, Glyph> = HashMap::new();

		let variants = match dto.variants {
			FontVariants::Single(variant) => vec![variant],
			FontVariants::Multiple { variants } => variants,
		};

		for (metrics_index, variant) in variants.into_iter().enumerate() {
			metrics.push(variant.metrics);
			for g in variant.glyphs.into_iter() {
				// If multiple variants contain the same unicode, last one wins
				glyphs.insert(
					g.unicode,
					Glyph {
						metrics_index,
						unicode: g.unicode,
						advance: g.advance,
						plane_bounds: g.plane_bounds,
						atlas_bounds: g.atlas_bounds,
					},
				);
			}
		}

		Font {
			atlas: dto.atlas,
			metrics,
			glyphs,
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
	pub metrics: Metrics,
	pub glyphs: Vec<GlyphDto>,
	// pub kerning: Vec<Kerning>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Atlas {
	pub r#type: Type,
	pub distance_range: i32,
	pub distance_range_middle: i32,
	pub size: f32,
	pub width: i32,
	pub height: i32,
	pub y_origin: YOrigin,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
	/// Monochrome (true) signed distance field.
	Sdf,
	/// Monochrome signed perpendicular distance field.
	Psdf,
	/// Multi-channel signed distance field.
	Msdf,
	/// Combined multi-channel and true signed distance field in the alpha channel.
	Mtsdf,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum YOrigin {
	Bottom,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
	pub em_size: f32,
	pub line_height: f32,
	pub ascender: f32,
	pub descender: f32,
	pub underline_y: f32,
	pub underline_thickness: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphDto {
	// When using -allglyphs this is the glyph index instead of unicode
	#[serde(alias = "index")]
	pub unicode: u32,
	pub advance: f32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub plane_bounds: Option<Bounds>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub atlas_bounds: Option<Bounds>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bounds {
	pub left: f32,
	pub bottom: f32,
	pub right: f32,
	pub top: f32,
}

// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Kerning {
// }

use super::*;
use cvmath::Vec2;

impl d2::IFont for Font {
	fn write_span(&self, mut cv: Option<&mut d2::TextBuffer>, scribe: &mut d2::Scribe, cursor: &mut Vec2<f32>, text: &str) {
		let font = self;
		let mut chars = text.chars();
		while let Some(chr) = chars.next() {
			if chr == '\n' {
				cursor.x = scribe.x_pos;
				cursor.y += scribe.line_height;
				continue;
			}

			// Process escape sequences
			if chr == '\x1b' {
				if chars.next() == Some('[') {
					if let Some((sequence, tail)) = chars.as_str().split_once("]") {
						d2::escape::process(sequence, scribe, match cv.as_mut() { Some(cv) => Some(*cv), None => None });
						chars = tail.chars();
					}
					else {
						// No terminal bracket found
						break;
					}
				}
				continue;
			}

			let Some(glyph) = font.glyphs.get(&(chr as u32)) else { continue };
			let metrics = &font.metrics[glyph.metrics_index];
			let pos = *cursor + Vec2(0.0, scribe.line_height - scribe.font_size - scribe.baseline);

			// Scale by per-glyph metrics em_size to support mixed variants
			let scale_x = scribe.font_size * scribe.font_width_scale / metrics.em_size;
			let scale_y = scribe.font_size / metrics.em_size;
			let advance = glyph.advance * scale_x + scribe.letter_spacing;
			cursor.x += advance;

			if !scribe.draw_mask {
				continue;
			}

			if let Some(cv) = &mut cv {
				let Some(plane_bounds) = &glyph.plane_bounds else { continue };
				let Some(atlas_bounds) = &glyph.atlas_bounds else { continue };

				let aleft = atlas_bounds.left;
				let aright = atlas_bounds.right;
				let atop = font.atlas.height as f32 - atlas_bounds.top;
				let abottom = font.atlas.height as f32 - atlas_bounds.bottom;

				let pleft = plane_bounds.left * scale_x;
				let pright = plane_bounds.right * scale_x;
				let ptop = (1.0 - plane_bounds.top) * scale_y;
				let pbottom = (1.0 - plane_bounds.bottom) * scale_y;

				let vertices = [
					d2::TextVertex {
						pos: pos + Vec2(pleft, pbottom),
						uv: Vec2(aleft, abottom) / Vec2(font.atlas.width as f32, font.atlas.height as f32),
						color: scribe.color,
						outline: scribe.outline,
					},
					d2::TextVertex {
						pos: pos + Vec2(pleft + scribe.top_skew, ptop),
						uv: Vec2(aleft, atop) / Vec2(font.atlas.width as f32, font.atlas.height as f32),
						color: scribe.color,
						outline: scribe.outline,
					},
					d2::TextVertex {
						pos: pos + Vec2(pright + scribe.top_skew, ptop),
						uv: Vec2(aright, atop) / Vec2(font.atlas.width as f32, font.atlas.height as f32),
						color: scribe.color,
						outline: scribe.outline,
					},
					d2::TextVertex {
						pos: pos + Vec2(pright, pbottom),
						uv: Vec2(aright, abottom) / Vec2(font.atlas.width as f32, font.atlas.height as f32),
						color: scribe.color,
						outline: scribe.outline,
					},
				];

				let mut p = cv.begin(PrimType::Triangles, 4, 2);
				p.add_indices_quad();
				p.add_vertices(&vertices);
			}
		}
	}
}
