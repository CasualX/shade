use std::collections::HashSet;
use std::fs;
use std::path::Path;

use shade::atlas::{self, Sprite};

type Result<T> = std::result::Result<T, ()>;
type Bounds = (i64, i64, i64, i64);

#[derive(Default)]
struct Stats {
	static_sprites: usize,
	animated_sprites: usize,
	frames: usize,
	glyphs: usize,
	visible_glyphs: usize,
	codepoints: usize,
	kerning_pairs: usize,
	regions: Vec<Bounds>,
}

pub fn run(path: &Path) -> Result<()> {
	let json = fs::read(path).map_err(|err| error!("failed to read atlas '{}': {err}", path.display()))?;
	let atlas: atlas::Atlas = serde_json::from_slice(&json)
		.map_err(|err| error!("failed to parse atlas '{}': {err}", path.display()))?;
	if atlas.meta.width <= 0 || atlas.meta.height <= 0 {
		fail!("atlas '{}' has invalid dimensions {}x{}", path.display(), atlas.meta.width, atlas.meta.height);
	}

	let stats = collect_stats(&atlas);
	let atlas_pixels = i64::from(atlas.meta.width) * i64::from(atlas.meta.height);
	let covered_pixels = union_area(&stats.regions, i64::from(atlas.meta.width), i64::from(atlas.meta.height));
	let coverage = covered_pixels as f64 * 100.0 / atlas_pixels as f64;
	let unique_regions = stats.regions.iter().copied().collect::<HashSet<_>>().len();
	let items = stats.frames + stats.glyphs;

	println!("Atlas:            {}", path.display());
	println!("Version:          {}", atlas.version);
	println!("Size:             {}x{} ({} pixels)", atlas.meta.width, atlas.meta.height, format_count(atlas_pixels));
	println!("Kind:             {}", atlas.meta.kind);
	if atlas.meta.distance_range != 0.0 || atlas.meta.distance_range_middle != 0.0 {
		println!("Distance range:   {} (center {})", atlas.meta.distance_range, atlas.meta.distance_range_middle);
	}
	println!("JSON size:        {}", format_bytes(json.len() as u64));
	println!("Items:            {} ({} sprite frames, {} glyphs)", format_count(items as i64), format_count(stats.frames as i64), format_count(stats.glyphs as i64));
	println!("Sprites:          {} ({} static, {} animated)", format_count(atlas.sprites.len() as i64), format_count(stats.static_sprites as i64), format_count(stats.animated_sprites as i64));
	println!("Fonts:            {} ({} visible glyphs, {} codepoints, {} kerning pairs)", format_count(atlas.fonts.len() as i64), format_count(stats.visible_glyphs as i64), format_count(stats.codepoints as i64), format_count(stats.kerning_pairs as i64));
	println!("Unique regions:   {}", format_count(unique_regions as i64));
	println!("Content coverage: {coverage:.1}% ({} / {} pixels)", format_count(covered_pixels), format_count(atlas_pixels));
	Ok(())
}

fn collect_stats(atlas: &atlas::Atlas) -> Stats {
	let mut stats = Stats::default();
	for sprite in atlas.sprites.values() {
		match sprite {
			Sprite::Frame(frame) => {
				stats.static_sprites += 1;
				stats.frames += 1;
				stats.regions.push(bounds(frame.rect.x, frame.rect.y, frame.rect.width, frame.rect.height));
			},
			Sprite::Animated(frames) => {
				stats.animated_sprites += 1;
				stats.frames += frames.len();
				stats.regions.extend(frames.iter().map(|frame| bounds(frame.frame.rect.x, frame.frame.rect.y, frame.frame.rect.width, frame.frame.rect.height)));
			},
		}
	}
	for font in atlas.fonts.values() {
		stats.glyphs += font.glyphs.len();
		stats.codepoints += font.codepoints.len();
		stats.kerning_pairs += font.kerning.len();
		for glyph in &font.glyphs {
			if let Some(bounds) = glyph.bounds {
				stats.visible_glyphs += 1;
				let rect = bounds.atlas_bounds;
				stats.regions.push(self::bounds(rect.x, rect.y, rect.width, rect.height));
			}
		}
	}
	stats
}

fn bounds(x: i32, y: i32, width: i32, height: i32) -> Bounds {
	(i64::from(x), i64::from(y), i64::from(x) + i64::from(width), i64::from(y) + i64::from(height))
}

/// Returns the union of all positive rectangles, clipped to the atlas bounds.
fn union_area(regions: &[Bounds], width: i64, height: i64) -> i64 {
	let regions = regions.iter().filter_map(|&(left, top, right, bottom)| {
		let clipped = (left.max(0), top.max(0), right.min(width), bottom.min(height));
		(clipped.0 < clipped.2 && clipped.1 < clipped.3).then_some(clipped)
	}).collect::<Vec<_>>();
	let mut xs = regions.iter().flat_map(|region| [region.0, region.2]).collect::<Vec<_>>();
	xs.sort_unstable();
	xs.dedup();

	let mut area = 0;
	for x_pair in xs.windows(2) {
		let (left, right) = (x_pair[0], x_pair[1]);
		let mut ys = regions.iter()
			.filter(|region| region.0 < right && region.2 > left)
			.map(|region| (region.1, region.3))
			.collect::<Vec<_>>();
		ys.sort_unstable();

		let mut covered_y = 0;
		let mut current: Option<(i64, i64)> = None;
		for (top, bottom) in ys {
			match current {
				Some((current_top, current_bottom)) if top <= current_bottom => current = Some((current_top, current_bottom.max(bottom))),
				Some((current_top, current_bottom)) => {
					covered_y += current_bottom - current_top;
					current = Some((top, bottom));
				},
				None => current = Some((top, bottom)),
			}
		}
		if let Some((top, bottom)) = current {
			covered_y += bottom - top;
		}
		area += (right - left) * covered_y;
	}
	area
}

fn format_count(value: i64) -> String {
	let digits = value.to_string();
	let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
	for (index, ch) in digits.chars().enumerate() {
		if index > 0 && (digits.len() - index).is_multiple_of(3) {
			formatted.push(',');
		}
		formatted.push(ch);
	}
	formatted
}

fn format_bytes(bytes: u64) -> String {
	if bytes < 1024 {
		format!("{bytes} B")
	}
	else if bytes < 1024 * 1024 {
		format!("{:.1} KiB", bytes as f64 / 1024.0)
	}
	else {
		format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn union_area_deduplicates_overlaps_and_clips() {
		let regions = [
			(0, 0, 4, 4),
			(0, 0, 4, 4),
			(2, 2, 6, 6),
			(8, 8, 12, 12),
		];
		assert_eq!(union_area(&regions, 10, 10), 32);
	}

	#[test]
	fn union_area_ignores_empty_and_outside_regions() {
		let regions = [(2, 2, 2, 5), (-5, -5, -1, -1), (1, 1, 3, 4)];
		assert_eq!(union_area(&regions, 10, 10), 6);
	}

	#[test]
	fn formats_counts_and_file_sizes() {
		assert_eq!(format_count(1_234_567), "1,234,567");
		assert_eq!(format_bytes(900), "900 B");
		assert_eq!(format_bytes(1536), "1.5 KiB");
	}
}
