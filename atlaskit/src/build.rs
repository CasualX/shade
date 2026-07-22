use std::collections::HashMap;
use std::ffi::OsStr;
use std::{env, fs, process, str, time};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use shade::cvmath::*;
use shade::{atlas, image, msdfgen};

mod manifest;

type Result<T> = std::result::Result<T, ()>;

#[derive(Copy, Clone, Debug)]
struct ViewBox {
	x: f64,
	y: f64,
	width: f64,
	height: f64,
}

fn resolve_sprite_processor(processor: Option<manifest::SpriteProcessor>, path: &Path) -> manifest::SpriteProcessor {
	match processor {
		None if has_extension(path, "svg") => manifest::SpriteProcessor::Msdfgen,
		None => manifest::SpriteProcessor::Blit,
		Some(processor) => processor,
	}
}

#[derive(Clone, Debug)]
struct AtlasSpec {
	kind: atlas::Kind,
	distance_range: f32,
	width: Option<i32>,
	height: Option<i32>,
	max_size: i32,
	recover_alpha_colors: bool,
	sprites: Vec<SpriteSpec>,
	fonts: Vec<FontSpec>,
	msdf_atlas_gen: MsdfAtlasGenSpec,
}

#[derive(Clone, Debug)]
pub struct BuildOptions {
	pub output: PathBuf,
	pub msdfgen: Option<PathBuf>,
	pub msdf_atlas_gen: Option<PathBuf>,
	pub temp_dir: Option<PathBuf>,
	pub keep_intermediate: bool,
	pub pretty: bool,
	pub preview: bool,
}

#[derive(Clone, Debug)]
enum ProcessorSpec {
	Blit,
	Msdfgen(MsdfgenSpec),
}

#[derive(Copy, Clone, Debug)]
struct MsdfgenSpec {
	mode: msdfgen::Mode,
	range: f32,
	size: f64,
	autoframe: bool,
	y_flip: bool,
	overlap: bool,
	fill_rule: msdfgen::FillRule,
}

#[derive(Copy, Clone, Debug)]
struct MsdfAtlasGenSpec {
	mode: msdfgen::Mode,
	range: f32,
	size: f64,
	outer_padding: f64,
	overlap: bool,
}

#[derive(Clone, Debug)]
struct SpriteSpec {
	name: String,
	path: PathBuf,
	duration: Option<f32>,
	processor: ProcessorSpec,
	margin: i32,
	gutter: manifest::GutterMode,
	transform: atlas::Transform,
	origin: Point2i,
}

#[derive(Clone, Debug)]
struct FontSpec {
	name: String,
	inputs: Vec<FontInputSpec>,
	margin: i32,
	msdf_atlas_gen: MsdfAtlasGenSpec,
}

#[derive(Clone, Debug)]
struct FontInputSpec {
	path: PathBuf,
	charset: FontCharset,
	font_scale: f64,
}

#[derive(Clone, Debug)]
enum FontCharset {
	Ascii,
	File(PathBuf),
	Inline(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum EntryProcessor {
	Blit {
		gutter: manifest::GutterMode,
	},
	Font {
		name: String,
	},
	Msdfgen {
		mode: msdfgen::Mode,
		range_bits: u32,
		size_bits: u64,
		autoframe: bool,
		y_flip: bool,
		overlap: bool,
		fill_rule: msdfgen::FillRule,
	},
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct EntryKey {
	path: PathBuf,
	processor: EntryProcessor,
	margin: i32,
}

struct Entry {
	key: EntryKey,
	image: image::ImageRGBA,
	content_width: i32,
	content_height: i32,
	rendered_gutter: bool,
}

impl Entry {
	fn occupied_size(&self) -> (i32, i32) {
		if self.rendered_gutter {
			(self.image.width, self.image.height)
		}
		else {
			(self.image.width + self.key.margin * 2, self.image.height + self.key.margin * 2)
		}
	}
}

struct ProcessedImage {
	image: image::ImageRGBA,
	content_width: i32,
	content_height: i32,
	rendered_gutter: bool,
}

struct LoadJob<'a> {
	key: EntryKey,
	sprite: &'a SpriteSpec,
	index: usize,
}

struct GeneratedFont {
	name: String,
	font: atlas::Font,
	glyph_entry_indices: Vec<Option<usize>>,
}

fn bitmap_gutter_mode(gutter: manifest::GutterMode) -> image::BlitGutterMode<[u8; 4]> {
	match gutter {
		manifest::GutterMode::ClampToEdge => image::BlitGutterMode::Edge,
		manifest::GutterMode::SelfTiled => image::BlitGutterMode::Repeat,
		manifest::GutterMode::Transparent => image::BlitGutterMode::Border([0, 0, 0, 0]),
	}
}

pub fn run(manifest: &Path, options: BuildOptions) -> Result<()> {
	let atlas = resolve_manifest(manifest)?;
	if atlas.sprites.is_empty() && atlas.fonts.is_empty() {
		fail!("manifest '{}' contains no sprites or fonts", manifest.display());
	}

	let needs_temp = !atlas.fonts.is_empty() || atlas.sprites.iter().any(|sprite| matches!(sprite.processor, ProcessorSpec::Msdfgen(_)));
	let temp_dir = needs_temp.then(|| create_temp_dir(&options)).transpose()?;
	let result = build_atlas(&atlas, &options, temp_dir.as_deref());

	if let Some(temp_dir) = temp_dir {
		if result.is_ok() && !options.keep_intermediate {
			fs::remove_dir_all(&temp_dir).map_err(|err| error!("failed to remove temporary directory '{}': {err}", temp_dir.display()))?;
		}
		else {
			eprintln!("warning: intermediate files kept at '{}'", temp_dir.display());
		}
	}
	result
}

fn resolve_manifest(path: &Path) -> Result<AtlasSpec> {
	if !has_extension(path, "ini") {
		fail!("manifest '{}' must be an INI file", path.display());
	}
	let manifest_path = path.canonicalize().map_err(|err| error!("failed to resolve manifest '{}': {err}", path.display()))?;
	let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
	let manifest_text = fs::read_to_string(&manifest_path).map_err(|err| error!("failed to read manifest '{}': {err}", manifest_path.display()))?;
	let manifest = manifest::Manifest::parse(&manifest_text).map_err(|err| error!("manifest '{}': {err}", manifest_path.display()))?;

	let msdfgen_mode = manifest.msdfgen.mode.unwrap_or(msdfgen::Mode::Mtsdf);
	let msdfgen_range = manifest.msdfgen.range.unwrap_or(4.0);
	let margin = manifest.margin.unwrap_or(1);
	let gutter = manifest.gutter.unwrap_or_default();
	let max_atlas_size = manifest.max_size.unwrap_or(4096);
	let size = manifest.msdfgen.size.unwrap_or(32.0);
	let autoframe = manifest.msdfgen.autoframe.unwrap_or(false);
	let y_flip = manifest.msdfgen.y_flip.unwrap_or(false);
	let overlap = manifest.msdfgen.overlap.unwrap_or(false);
	let fill_rule = manifest.msdfgen.fill_rule.unwrap_or_default();
	let msdf_atlas_gen = MsdfAtlasGenSpec {
		mode: manifest.msdf_atlas_gen.mode.unwrap_or(msdfgen::Mode::Mtsdf),
		range: manifest.msdf_atlas_gen.range.unwrap_or(4.0),
		size: manifest.msdf_atlas_gen.size.unwrap_or(32.0),
		outer_padding: manifest.msdf_atlas_gen.outer_padding.unwrap_or(1.0),
		overlap: manifest.msdf_atlas_gen.overlap.unwrap_or(false),
	};
	let mut atlas = AtlasSpec {
		kind: manifest.kind.unwrap_or(atlas::Kind::Bitmap),
		distance_range: manifest.distance_range.unwrap_or(0.0),
		width: manifest.width,
		height: manifest.height,
		max_size: max_atlas_size,
		recover_alpha_colors: manifest.recover_alpha_colors.unwrap_or(false),
		sprites: Vec::with_capacity(manifest.sprites.len()),
		fonts: Vec::new(),
		msdf_atlas_gen,
	};
	validate_defaults(&manifest_path, &atlas, margin, msdfgen_range)?;

	validate_sprite_durations(&manifest_path, &manifest.sprites)?;
	for sprite in manifest.sprites {
		let path = sprite.path.as_deref().ok_or_else(|| error!("manifest '{}' section [Sprite:{}] has no Path", manifest_path.display(), sprite.name))?;
		let path = resolve_relative(base, path).canonicalize().map_err(|err| error!("failed to resolve Path for [Sprite:{}] in '{}': {err}", sprite.name, manifest_path.display()))?;
		let processor = match resolve_sprite_processor(sprite.processor.or(manifest.sprite_processor), &path) {
			manifest::SpriteProcessor::Blit => ProcessorSpec::Blit,
			manifest::SpriteProcessor::Msdfgen => ProcessorSpec::Msdfgen(MsdfgenSpec {
				mode: msdfgen_mode,
				range: msdfgen_range,
				size: sprite.msdfgen.size.unwrap_or(size),
				autoframe: sprite.msdfgen.autoframe.unwrap_or(autoframe),
				y_flip: sprite.msdfgen.y_flip.unwrap_or(y_flip),
				overlap: sprite.msdfgen.overlap.unwrap_or(overlap),
				fill_rule: sprite.msdfgen.fill_rule.unwrap_or(fill_rule),
			}),
		};
		let spec = SpriteSpec {
			name: sprite.name,
			path,
			duration: sprite.duration,
			processor,
			margin: sprite.margin.unwrap_or(margin),
			gutter: sprite.gutter.unwrap_or(gutter),
			transform: sprite.transform.unwrap_or_default(),
			origin: sprite.origin.unwrap_or(Point2i(0, 0)),
		};
		validate_sprite(&manifest_path, &spec)?;
		atlas.sprites.push(spec);
	}
	warn_on_msdfgen_metadata_mismatch(&manifest_path, &atlas, msdfgen_mode, msdfgen_range);

	let mut font_indices = HashMap::<String, usize>::new();
	for font in manifest.fonts {
		let path = font.path.as_deref().ok_or_else(|| error!("manifest '{}' section [Font:{}] has no Path", manifest_path.display(), font.name))?;
		let path = resolve_relative(base, path).canonicalize().map_err(|err| error!("failed to resolve Path for [Font:{}] in '{}': {err}", font.name, manifest_path.display()))?;
		let selected_charsets = usize::from(font.msdf_atlas_gen.charset.is_some()) + usize::from(font.msdf_atlas_gen.chars.is_some());
		if selected_charsets > 1 {
			fail!("manifest '{}' [Font:{}]: MsdfAtlasGen.Charset and MsdfAtlasGen.Chars are mutually exclusive", manifest_path.display(), font.name);
		}
		let charset = if let Some(path) = font.msdf_atlas_gen.charset {
			let path = resolve_relative(base, &path).canonicalize().map_err(|err| error!("failed to resolve MsdfAtlasGen.Charset for [Font:{}] in '{}': {err}", font.name, manifest_path.display()))?;
			FontCharset::File(path)
		}
		else if let Some(chars) = font.msdf_atlas_gen.chars {
			FontCharset::Inline(chars)
		}
		else {
			FontCharset::Ascii
		};
		let input = FontInputSpec { path, charset, font_scale: font.font_scale.unwrap_or(1.0) };
		let font_msdf_atlas_gen = MsdfAtlasGenSpec {
			size: font.msdf_atlas_gen.size.unwrap_or(msdf_atlas_gen.size),
			outer_padding: font.msdf_atlas_gen.outer_padding.unwrap_or(msdf_atlas_gen.outer_padding),
			overlap: font.msdf_atlas_gen.overlap.unwrap_or(msdf_atlas_gen.overlap),
			..msdf_atlas_gen
		};
		let font_margin = font.margin.unwrap_or(margin);
		if let Some(&index) = font_indices.get(&font.name) {
			let existing = &mut atlas.fonts[index];
			if (existing.margin, existing.msdf_atlas_gen.size.to_bits(), existing.msdf_atlas_gen.outer_padding.to_bits(), existing.msdf_atlas_gen.overlap) != (font_margin, font_msdf_atlas_gen.size.to_bits(), font_msdf_atlas_gen.outer_padding.to_bits(), font_msdf_atlas_gen.overlap) {
				fail!("manifest '{}' repeated [Font:{}] sections must use the same Margin, MsdfAtlasGen.Size, MsdfAtlasGen.OuterPadding, and MsdfAtlasGen.Overlap", manifest_path.display(), font.name);
			}
			existing.inputs.push(input);
		}
		else {
			let index = atlas.fonts.len();
			font_indices.insert(font.name.clone(), index);
			atlas.fonts.push(FontSpec { name: font.name, inputs: vec![input], margin: font_margin, msdf_atlas_gen: font_msdf_atlas_gen });
		}
	}
	for font in &atlas.fonts {
		validate_font(&manifest_path, font)?;
	}

	Ok(atlas)
}

fn warn_on_msdfgen_metadata_mismatch(manifest: &Path, atlas_spec: &AtlasSpec, mode: msdfgen::Mode, range: f32) {
	if !atlas_spec.sprites.iter().any(|sprite| matches!(sprite.processor, ProcessorSpec::Msdfgen(_))) {
		return;
	}

	let (kind_mismatch, range_mismatch) = msdfgen_metadata_mismatches(atlas_spec.kind, atlas_spec.distance_range, mode, range);
	if kind_mismatch {
		eprintln!("warning: manifest '{}': Kind={} disagrees with Msdfgen.Mode={}", manifest.display(), atlas_spec.kind, mode.as_str());
	}
	if range_mismatch {
		eprintln!("warning: manifest '{}': DistanceRange={} disagrees with Msdfgen.Range={range}", manifest.display(), atlas_spec.distance_range);
	}
}

fn msdfgen_metadata_mismatches(kind: atlas::Kind, distance_range: f32, mode: msdfgen::Mode, range: f32) -> (bool, bool) {
	(kind != atlas::Kind::from(mode), distance_range != range)
}

fn validate_defaults(manifest: &Path, atlas: &AtlasSpec, margin: i32, msdfgen_range: f32) -> Result<()> {
	if margin < 0 {
		fail!("manifest '{}': Margin must be non-negative", manifest.display());
	}
	let has_dynamic_dimension = atlas.width.is_none() || atlas.height.is_none();
	if has_dynamic_dimension && atlas.max_size <= 0 {
		fail!("manifest '{}': MaxSize must be positive", manifest.display());
	}
	if atlas.width.is_some_and(|value| value <= 0) || atlas.height.is_some_and(|value| value <= 0) {
		fail!("manifest '{}': atlas dimensions must be positive", manifest.display());
	}
	if !atlas.distance_range.is_finite() || atlas.distance_range < 0.0 {
		fail!("manifest '{}': DistanceRange must be a non-negative finite number", manifest.display());
	}
	if !msdfgen_range.is_finite() || msdfgen_range <= 0.0 {
		fail!("manifest '{}': Msdfgen.Range must be a positive finite number", manifest.display());
	}
	if !atlas.msdf_atlas_gen.range.is_finite() || atlas.msdf_atlas_gen.range <= 0.0 {
		fail!("manifest '{}': MsdfAtlasGen.Range must be a positive finite number", manifest.display());
	}
	Ok(())
}

fn validate_sprite_durations(manifest: &Path, sprites: &[manifest::Sprite]) -> Result<()> {
	let mut frame_counts = HashMap::<&str, usize>::new();
	for sprite in sprites {
		*frame_counts.entry(&sprite.name).or_default() += 1;
	}

	for sprite in sprites {
		let animated = frame_counts[sprite.name.as_str()] > 1;
		match (animated, sprite.duration) {
			(true, None) => fail!("manifest '{}' repeated [Sprite:{}] sections require Duration", manifest.display(), sprite.name),
			(false, Some(_)) => fail!("manifest '{}' [Sprite:{}]: Duration is only valid for repeated sprite names", manifest.display(), sprite.name),
			_ => {},
		}
	}
	Ok(())
}

fn validate_sprite(manifest: &Path, sprite: &SpriteSpec) -> Result<()> {
	if sprite.margin < 0 {
		fail!("manifest '{}' [Sprite:{}]: Margin must be non-negative", manifest.display(), sprite.name);
	}
	match &sprite.processor {
		ProcessorSpec::Blit if has_extension(&sprite.path, "svg") => {
			fail!("manifest '{}' [Sprite:{}] cannot blit SVG Path '{}'; use Processor=msdfgen", manifest.display(), sprite.name, sprite.path.display());
		},
		ProcessorSpec::Msdfgen(_) if !has_extension(&sprite.path, "svg") => {
			fail!("manifest '{}' [Sprite:{}] uses msdfgen, but Path '{}' is not an SVG", manifest.display(), sprite.name, sprite.path.display());
		},
		ProcessorSpec::Msdfgen(settings) if !settings.size.is_finite() || settings.size <= 0.0 => {
			fail!("manifest '{}' [Sprite:{}]: Msdfgen.Size must be a positive finite number", manifest.display(), sprite.name);
		},
		ProcessorSpec::Msdfgen(settings) if !settings.range.is_finite() || settings.range <= 0.0 => {
			fail!("manifest '{}' [Sprite:{}]: Msdfgen.Range must be a positive finite number", manifest.display(), sprite.name);
		},
		ProcessorSpec::Msdfgen(settings) if settings.autoframe && sprite.margin != 0 => {
			fail!("manifest '{}' [Sprite:{}]: Msdfgen.Autoframe=true requires Margin=0", manifest.display(), sprite.name);
		},
		_ => {},
	}
	Ok(())
}

fn validate_font(manifest: &Path, font: &FontSpec) -> Result<()> {
	if font.margin < 0 {
		fail!("manifest '{}' [Font:{}]: Margin must be non-negative", manifest.display(), font.name);
	}
	if !font.msdf_atlas_gen.size.is_finite() || font.msdf_atlas_gen.size <= 0.0 {
		fail!("manifest '{}' [Font:{}]: MsdfAtlasGen.Size must be a positive finite number", manifest.display(), font.name);
	}
	if !font.msdf_atlas_gen.outer_padding.is_finite() || font.msdf_atlas_gen.outer_padding < 0.0 {
		fail!("manifest '{}' [Font:{}]: MsdfAtlasGen.OuterPadding must be a non-negative finite number", manifest.display(), font.name);
	}
	for input in &font.inputs {
		if !has_extension(&input.path, "ttf") && !has_extension(&input.path, "otf") {
			fail!("manifest '{}' [Font:{}] Path '{}' must be a TTF or OTF file", manifest.display(), font.name, input.path.display());
		}
		if !input.font_scale.is_finite() || input.font_scale <= 0.0 {
			fail!("manifest '{}' [Font:{}]: FontScale must be a positive finite number", manifest.display(), font.name);
		}
	}
	Ok(())
}

fn build_atlas(atlas_spec: &AtlasSpec, options: &BuildOptions, temp_dir: Option<&Path>) -> Result<()> {
	let (load_jobs, sprite_entry_indices) = plan_loads(&atlas_spec.sprites);
	let mut entries = load_entries(options, load_jobs, temp_dir)?;
	let (font_entries, mut generated_fonts) = generate_fonts(atlas_spec, options, temp_dir)?;
	let first_font_entry = entries.len();
	for font in &mut generated_fonts {
		for entry_index in font.glyph_entry_indices.iter_mut().flatten() {
			*entry_index += first_font_entry;
		}
	}
	entries.extend(font_entries);

	// Loading is the only parallel phase.
	// Packing and atlas composition remain serial so their output stays deterministic.
	let (width, height, positions) = pack_entries(atlas_spec, &entries)?;
	let mut image = image::ImageRGBA::new(width, height, [0, 0, 0, 0]);
	let mut entry_frames = Vec::with_capacity(entries.len());
	for (entry, &position) in entries.iter().zip(&positions) {
		entry_frames.push(atlas::Frame {
			rect: Recti(position.x, position.y, entry.content_width, entry.content_height),
			margin: entry.key.margin,
			transform: atlas::Transform::None,
			origin: Point2i(0, 0),
		});
	}
	for (entry, &position) in entries.iter().zip(&positions) {
		if entry.rendered_gutter {
			image.copy_from(Point2i(position.x - entry.key.margin, position.y - entry.key.margin), &entry.image);
		}
		else {
			let gutter = match &entry.key.processor {
				EntryProcessor::Blit { gutter } => bitmap_gutter_mode(*gutter),
				EntryProcessor::Font { .. } => image::BlitGutterMode::Edge,
				EntryProcessor::Msdfgen { .. } => unreachable!("msdfgen renders its own gutter"),
			};
			image.copy_with_gutter(position, &entry.image, entry.key.margin, gutter);
		}
	}
	image = apply_alpha_color_recovery(image, atlas_spec.recover_alpha_colors);

	let sprites = build_sprites(&atlas_spec.sprites, &sprite_entry_indices, &entry_frames);
	let mut fonts = HashMap::with_capacity(generated_fonts.len());
	for GeneratedFont { name, font, glyph_entry_indices } in generated_fonts {
		fonts.insert(name, relocate_font(font, &glyph_entry_indices, &positions, (width, height)));
	}
	let atlas = atlas::Atlas::<String, String> {
		version: 0,
		meta: atlas::Metadata {
			width,
			height,
			kind: atlas_spec.kind,
			distance_range: atlas_spec.distance_range,
			distance_range_middle: 0.0,
		},
		sprites,
		fonts,
	};

	let png_path = options.output.with_added_extension("png");
	let json_path = options.output.with_added_extension("json");
	if let Some(parent) = png_path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
		fs::create_dir_all(parent).map_err(|err| error!("failed to create output directory '{}': {err}", parent.display()))?;
	}
	image.save_file_png(&png_path).map_err(|err| error!("failed to write atlas PNG '{}': {err}", png_path.display()))?;
	let json = if options.pretty { serde_json::to_string_pretty(&atlas) } else { serde_json::to_string(&atlas) }.unwrap();
	fs::write(&json_path, json).map_err(|err| error!("failed to write atlas metadata '{}': {err}", json_path.display()))?;
	println!("packed {} images for {} sprites and {} fonts into {}x{}", entries.len(), atlas.sprites.len(), atlas_spec.fonts.len(), width, height);
	println!("wrote {}", png_path.display());
	println!("wrote {}", json_path.display());

	if options.preview {
		let preview_path = options.output.with_added_extension("preview.png");
		render_preview(&image, atlas_spec.kind, atlas_spec.distance_range)
			.save_file_png(&preview_path)
			.map_err(|err| error!("failed to write atlas preview '{}': {err}", preview_path.display()))?;
		println!("wrote {}", preview_path.display());
	}
	Ok(())
}

fn apply_alpha_color_recovery(image: image::ImageRGBA, enabled: bool) -> image::ImageRGBA {
	if enabled { image.recover_alpha_colors() } else { image }
}

fn build_sprites(sprite_specs: &[SpriteSpec], sprite_entry_indices: &[usize], entry_frames: &[atlas::Frame]) -> HashMap<String, atlas::Sprite> {
	assert_eq!(sprite_specs.len(), sprite_entry_indices.len());
	let mut sprites = HashMap::with_capacity(sprite_specs.len());
	for (sprite, &index) in sprite_specs.iter().zip(sprite_entry_indices) {
		let mut frame = entry_frames[index].clone();
		frame.transform = sprite.transform;
		frame.origin = sprite.origin;
		if let Some(duration) = sprite.duration {
			let animated_frame = atlas::AnimatedFrame { frame, duration };
			match sprites.entry(sprite.name.clone()) {
				std::collections::hash_map::Entry::Vacant(entry) => {
					entry.insert(atlas::Sprite::Animated(vec![animated_frame]));
				},
				std::collections::hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
					atlas::Sprite::Animated(frames) => frames.push(animated_frame),
					atlas::Sprite::Frame(_) => unreachable!("validated animated sprite collided with a static sprite"),
				},
			}
		}
		else {
			let previous = sprites.insert(sprite.name.clone(), atlas::Sprite::Frame(frame));
			assert!(previous.is_none(), "validated static sprite name was repeated");
		}
	}
	sprites
}

fn relocate_font(mut font: atlas::Font, glyph_entry_indices: &[Option<usize>], positions: &[Point2i], atlas_dimensions: (i32, i32)) -> atlas::Font {
	assert_eq!(font.glyphs.len(), glyph_entry_indices.len());
	for (glyph, entry_index) in font.glyphs.iter_mut().zip(glyph_entry_indices) {
		let (Some(bounds), Some(entry_index)) = (&mut glyph.bounds, entry_index) else { continue };
		let position = positions[*entry_index];
		bounds.atlas_bounds.x = position.x;
		bounds.atlas_bounds.y = position.y;
	}
	font.meta.width = atlas_dimensions.0;
	font.meta.height = atlas_dimensions.1;
	font
}

fn generate_fonts(atlas: &AtlasSpec, options: &BuildOptions, temp_dir: Option<&Path>) -> Result<(Vec<Entry>, Vec<GeneratedFont>)> {
	if atlas.fonts.is_empty() {
		return Ok((Vec::new(), Vec::new()));
	}
	let msdf_atlas_gen = options.msdf_atlas_gen.as_deref().ok_or_else(|| error!("fonts require the --msdf-atlas-gen PROGRAM argument"))?;
	let temp_dir = temp_dir.ok_or_else(|| error!("internal error: no temporary directory for msdf-atlas-gen"))?;
	let mut entries = Vec::with_capacity(atlas.fonts.len());
	let mut fonts = Vec::with_capacity(atlas.fonts.len());
	for (index, spec) in atlas.fonts.iter().enumerate() {
		let generated_image = temp_dir.join(format!("font-{index:04}.png"));
		let generated_json = temp_dir.join(format!("font-{index:04}.json"));
		run_msdf_atlas_gen(spec, msdf_atlas_gen, &generated_image, &generated_json)?;
		let image = image::ImageRGBA::load_file_png(&generated_image).map_err(|err| error!("failed to load generated font PNG '{}': {err}", generated_image.display()))?;
		let json = fs::read(&generated_json).map_err(|err| error!("failed to read generated font JSON '{}': {err}", generated_json.display()))?;
		let dto = serde_json::from_slice::<msdfgen::FontDto>(&json).map_err(|err| error!("failed to parse generated font JSON '{}': {err}", generated_json.display()))?;
		let font = atlas::Font::from(dto);
		if font.meta.width != image.width || font.meta.height != image.height {
			fail!("msdf-atlas-gen metadata for [Font:{}] says {}x{}, but its image is {}x{}", spec.name, font.meta.width, font.meta.height, image.width, image.height);
		}
		if font.meta.kind != spec.msdf_atlas_gen.mode.into() {
			fail!("msdf-atlas-gen generated {} data for [Font:{}], expected {}", font.meta.kind, spec.name, atlas::Kind::from(spec.msdf_atlas_gen.mode));
		}
		if font.meta.distance_range != spec.msdf_atlas_gen.range || font.meta.distance_range_middle != 0.0 {
			fail!(
				"msdf-atlas-gen generated distance range {} centered at {} for [Font:{}], expected {} centered at 0",
				font.meta.distance_range,
				font.meta.distance_range_middle,
				spec.name,
				spec.msdf_atlas_gen.range,
			);
		}
		let doubled_margin = spec.margin.checked_mul(2).ok_or_else(|| error!("[Font:{}] Margin is too large", spec.name))?;
		let mut glyph_entry_indices = Vec::with_capacity(font.glyphs.len());
		let first_glyph_entry = entries.len();
		for (glyph_index, glyph) in font.glyphs.iter().enumerate() {
			let Some(bounds) = glyph.bounds else {
				glyph_entry_indices.push(None);
				continue;
			};
			let rect = bounds.atlas_bounds;
			if rect.width <= 0 || rect.height <= 0 {
				fail!("msdf-atlas-gen produced invalid glyph rectangle {:?} for glyph {glyph_index} in [Font:{}]", rect, spec.name);
			}
			let glyph_image = image.sub_image(rect.x, rect.y, rect.width, rect.height).ok_or_else(|| {
				error!("msdf-atlas-gen glyph rectangle {:?} for glyph {glyph_index} in [Font:{}] exceeds its {}x{} image", rect, spec.name, image.width, image.height)
			})?;
			rect.width.checked_add(doubled_margin).ok_or_else(|| error!("[Font:{}] occupied glyph width overflows", spec.name))?;
			rect.height.checked_add(doubled_margin).ok_or_else(|| error!("[Font:{}] occupied glyph height overflows", spec.name))?;
			let entry_index = entries.len();
			entries.push(Entry {
				key: EntryKey { path: spec.inputs[0].path.clone(), processor: EntryProcessor::Font { name: spec.name.clone() }, margin: spec.margin },
				image: glyph_image,
				content_width: rect.width,
				content_height: rect.height,
				rendered_gutter: false,
			});
			glyph_entry_indices.push(Some(entry_index));
		}
		println!("generated {:>4}x{:<4} font {} as {} glyph images", image.width, image.height, spec.name, entries.len() - first_glyph_entry);
		fonts.push(GeneratedFont { name: spec.name.clone(), font, glyph_entry_indices });
	}
	Ok((entries, fonts))
}

fn run_msdf_atlas_gen(font: &FontSpec, executable: &Path, image_output: &Path, json_output: &Path) -> Result<()> {
	let mut command = command_for(executable);
	for (index, input) in font.inputs.iter().enumerate() {
		if index != 0 {
			command.arg("-and");
		}
		command.arg("-font").arg(&input.path);
		match &input.charset {
			FontCharset::Ascii => {},
			FontCharset::File(path) => { command.arg("-charset").arg(path); },
			FontCharset::Inline(chars) => { command.arg("-chars").arg(chars); },
		}
		if input.font_scale != 1.0 {
			command.arg("-fontscale").arg(input.font_scale.to_string());
		}
	}
	command
		.arg("-type").arg(font.msdf_atlas_gen.mode.as_str())
		.arg("-format").arg("png")
		.arg("-potr")
		.arg("-imageout").arg(image_output)
		.arg("-json").arg(json_output)
		.arg("-size").arg(font.msdf_atlas_gen.size.to_string())
		.arg("-pxrange").arg(font.msdf_atlas_gen.range.to_string())
		.arg("-outerpxpadding").arg(font.msdf_atlas_gen.outer_padding.to_string());
	if font.msdf_atlas_gen.overlap {
		command.arg("-overlap");
	}

	let result = command.output().map_err(|err| error!("failed to launch msdf-atlas-gen for [Font:{}]: {err}", font.name))?;
	if !result.status.success() {
		let stdout = String::from_utf8_lossy(&result.stdout);
		let stderr = String::from_utf8_lossy(&result.stderr);
		fail!("msdf-atlas-gen failed for [Font:{}] with {}\nstdout:\n{}\nstderr:\n{}", font.name, result.status, stdout.trim(), stderr.trim());
	}
	Ok(())
}

fn plan_loads(sprite_specs: &[SpriteSpec]) -> (Vec<LoadJob<'_>>, Vec<usize>) {
	let mut entry_indices = HashMap::<EntryKey, usize>::new();
	let mut jobs = Vec::new();
	let mut sprite_entry_indices = Vec::with_capacity(sprite_specs.len());

	for sprite in sprite_specs {
		let key = EntryKey {
			path: sprite.path.clone(),
			processor: match &sprite.processor {
				ProcessorSpec::Blit => EntryProcessor::Blit { gutter: sprite.gutter },
				ProcessorSpec::Msdfgen(settings) => EntryProcessor::Msdfgen {
					mode: settings.mode,
					range_bits: settings.range.to_bits(),
					size_bits: settings.size.to_bits(),
					autoframe: settings.autoframe,
					y_flip: settings.y_flip,
					overlap: settings.overlap,
					fill_rule: settings.fill_rule,
				},
			},
			margin: sprite.margin,
		};
		let entry_index = if let Some(&index) = entry_indices.get(&key) {
			index
		}
		else {
			let index = jobs.len();
			entry_indices.insert(key.clone(), index);
			jobs.push(LoadJob { key, sprite, index });
			index
		};
		sprite_entry_indices.push(entry_index);
	}
	(jobs, sprite_entry_indices)
}

fn load_entries(options: &BuildOptions, jobs: Vec<LoadJob<'_>>, temp_dir: Option<&Path>) -> Result<Vec<Entry>> {
	let results: Vec<_> = jobs
		.into_par_iter()
		.map(|job| -> Result<Entry> {
			let processed = match &job.sprite.processor {
				ProcessorSpec::Blit => process_bitmap(job.sprite),
				ProcessorSpec::Msdfgen(settings) => process_msdfgen(options, job.sprite, settings, job.index, temp_dir),
			}?;
			if processed.content_width <= 0 || processed.content_height <= 0 {
				fail!("Path '{}' produced an empty image", job.sprite.path.display());
			}
			Ok(Entry {
				key: job.key,
				image: processed.image,
				content_width: processed.content_width,
				content_height: processed.content_height,
				rendered_gutter: processed.rendered_gutter,
			})
		})
		.collect();

	// Indexed parallel collection restores manifest order. Report successful
	// work serially so its output stays deterministic.
	let entries = results.into_iter().collect::<Result<Vec<_>>>()?;
	for entry in &entries {
		match &entry.key.processor {
			EntryProcessor::Blit { .. } => println!("loaded    {:>4}x{:<4} {}", entry.content_width, entry.content_height, entry.key.path.display()),
			EntryProcessor::Font { name } => println!("generated {:>4}x{:<4} font {name}", entry.content_width, entry.content_height),
			EntryProcessor::Msdfgen { .. } => println!(
				"generated {:>4}x{:<4} {} ({}px rendered gutter)",
				entry.image.width,
				entry.image.height,
				entry.key.path.display(),
				entry.key.margin,
			),
		}
	}
	Ok(entries)
}

fn process_msdfgen(options: &BuildOptions, sprite: &SpriteSpec, settings: &MsdfgenSpec, index: usize, temp_dir: Option<&Path>) -> Result<ProcessedImage> {
	let executable = options.msdfgen.as_deref().ok_or_else(|| error!("[Sprite:{}] requires the --msdfgen PROGRAM argument", sprite.name))?;
	let temp_dir = temp_dir.ok_or_else(|| error!("internal error: no temporary directory for msdfgen"))?;
	let view_box = parse_view_box(&sprite.path)?;
	let scale = settings.size / 32.0;
	let width = (view_box.width * scale).ceil() as i32;
	let height = (view_box.height * scale).ceil() as i32;
	if width <= 0 || height <= 0 {
		fail!("SVG '{}' produced invalid dimensions {width}x{height}", sprite.path.display());
	}
	let doubled_margin = sprite.margin.checked_mul(2).ok_or_else(|| error!("[Sprite:{}] Margin is too large", sprite.name))?;
	let render_width = width.checked_add(doubled_margin).ok_or_else(|| error!("[Sprite:{}] rendered width overflows", sprite.name))?;
	let render_height = height.checked_add(doubled_margin).ok_or_else(|| error!("[Sprite:{}] rendered height overflows", sprite.name))?;
	let stem = sprite.path.file_stem().and_then(OsStr::to_str).unwrap_or("sprite");
	let generated = temp_dir.join(format!("{index:04}-{stem}.png"));
	run_msdfgen(sprite, settings, executable, &generated, view_box, (render_width, render_height))?;
	let image = image::ImageRGBA::load_file_png(&generated).map_err(|err| error!("failed to load generated PNG '{}': {err}", generated.display()))?;
	if image.width != render_width || image.height != render_height {
		fail!("msdfgen produced {}x{} for '{}', expected {render_width}x{render_height}", image.width, image.height, sprite.path.display());
	}
	Ok(ProcessedImage { image, content_width: width, content_height: height, rendered_gutter: true })
}

fn process_bitmap(sprite: &SpriteSpec) -> Result<ProcessedImage> {
	let image = image::ImageRGBA::load_file(&sprite.path).map_err(|err| error!("failed to load Path '{}' for [Sprite:{}]: {err}", sprite.path.display(), sprite.name))?;
	let doubled_margin = sprite.margin.checked_mul(2).ok_or_else(|| error!("[Sprite:{}] Margin is too large", sprite.name))?;
	image.width.checked_add(doubled_margin).ok_or_else(|| error!("[Sprite:{}] occupied width overflows", sprite.name))?;
	image.height.checked_add(doubled_margin).ok_or_else(|| error!("[Sprite:{}] occupied height overflows", sprite.name))?;
	Ok(ProcessedImage { content_width: image.width, content_height: image.height, image, rendered_gutter: false })
}

fn create_temp_dir(options: &BuildOptions) -> Result<PathBuf> {
	let path = match &options.temp_dir {
		Some(path) => path.clone(),
		None => {
			let nonce = time::SystemTime::now()
				.duration_since(time::UNIX_EPOCH)
				.map_err(|err| error!("system clock is before the Unix epoch: {err}"))?
				.as_millis();
			env::temp_dir().join(format!("atlaskit-{}-{nonce}", process::id()))
		},
	};
	fs::create_dir(&path).map_err(|err| error!("failed to create new temporary directory '{}': {err}", path.display()))?;
	Ok(path)
}

fn run_msdfgen(sprite: &SpriteSpec, settings: &MsdfgenSpec, executable: &Path, output: &Path, view_box: ViewBox, dimensions: (i32, i32)) -> Result<()> {
	let (width, height) = dimensions;
	let scale = settings.size / 32.0;
	let mut command = command_for(executable);
	command
		.arg(settings.mode.as_str())
		.arg("-svg").arg(&sprite.path)
		.arg("-o").arg(output)
		.arg("-dimensions").arg(width.to_string()).arg(height.to_string())
		.arg("-pxrange").arg(settings.range.to_string())
		.arg("-fillrule").arg(settings.fill_rule.as_str());
	if settings.autoframe {
		command.arg("-autoframe");
	}
	else {
		let margin = f64::from(sprite.margin) / scale;
		command.arg("-scale").arg(scale.to_string()).arg("-translate").arg((-view_box.x + margin).to_string()).arg((-view_box.y + margin).to_string());
	}
	if settings.y_flip {
		command.arg("-yflip");
	}
	if settings.overlap {
		command.arg("-overlap");
	}

	let result = command.output().map_err(|err| error!("failed to launch msdfgen for '{}': {err}", sprite.path.display()))?;
	if !result.status.success() {
		fail!(
			"msdfgen failed for '{}' with {}\nstdout:\n{}\nstderr:\n{}",
			sprite.path.display(),
			result.status,
			String::from_utf8_lossy(&result.stdout).trim(),
			String::from_utf8_lossy(&result.stderr).trim(),
		);
	}
	Ok(())
}

fn command_for(program: &Path) -> process::Command {
	#[cfg(target_os = "linux")]
	if program.extension() == Some(OsStr::new("exe")) {
		let mut command = process::Command::new("wine");
		command.arg(program);
		return command;
	}
	process::Command::new(program)
}

fn parse_view_box(path: &Path) -> Result<ViewBox> {
	let svg = fs::read_to_string(path).map_err(|err| error!("failed to read SVG '{}': {err}", path.display()))?;
	parse_view_box_source(path, &svg)
}

fn parse_view_box_source(path: &Path, svg: &str) -> Result<ViewBox> {
	let document = tagsoup::Document::parse(svg);
	let element = document.children.iter().filter_map(|node| node.element()).find(|element| element.tag == "svg").ok_or_else(|| error!("SVG '{}' has no root svg element", path.display()))?;
	let view_box = element.get_attribute_value("viewBox").ok_or_else(|| error!("SVG '{}' has no viewBox", path.display()))?;
	let values = view_box
		.split(|character: char| character.is_ascii_whitespace() || character == ',')
		.filter(|value| !value.is_empty())
		.map(str::parse::<f64>)
		.collect::<std::result::Result<Vec<_>, _>>()
		.map_err(|err| error!("invalid viewBox in '{}': {err}", path.display()))?;
	if values.len() != 4 || values.iter().any(|value| !value.is_finite()) || values[2] <= 0.0 || values[3] <= 0.0 {
		fail!("invalid viewBox in '{}': expected four finite values and a positive size", path.display());
	}
	Ok(ViewBox { x: values[0], y: values[1], width: values[2], height: values[3] })
}

fn pack_entries(atlas: &AtlasSpec, entries: &[Entry]) -> Result<(i32, i32, Vec<Point2i>)> {
	let total_area = entries.iter().try_fold(0_i64, |area, entry| {
		let (width, height) = entry.occupied_size();
		let image_area = i64::from(width).checked_mul(i64::from(height)).ok_or_else(|| error!("image area overflow"))?;
		area.checked_add(image_area).ok_or_else(|| error!("combined image area overflow"))
	})?;
	let largest_width = entries.iter().map(|entry| entry.occupied_size().0).max().unwrap_or(1);
	let largest_height = entries.iter().map(|entry| entry.occupied_size().1).max().unwrap_or(1);
	let target_area = (total_area as f64 * 1.25).ceil() as i64;
	let estimated_side = next_power_of_two((target_area as f64).sqrt().ceil() as i32);
	let estimated_half = estimated_side / 2;
	let can_use_half = i64::from(estimated_side) * i64::from(estimated_half) >= target_area;
	let total_width = entries.iter().map(|entry| i64::from(entry.occupied_size().0)).sum::<i64>();
	let total_height = entries.iter().map(|entry| i64::from(entry.occupied_size().1)).sum::<i64>();
	let (estimated_width, estimated_height) = if can_use_half && estimated_side >= largest_width && estimated_half >= largest_height && total_width >= total_height {
		(estimated_side, estimated_half)
	}
	else if can_use_half && estimated_half >= largest_width && estimated_side >= largest_height {
		(estimated_half, estimated_side)
	}
	else {
		(estimated_side.max(largest_width), estimated_side.max(largest_height))
	};
	let mut width = atlas.width.unwrap_or(estimated_width);
	let mut height = atlas.height.unwrap_or(estimated_height);

	loop {
		if (atlas.width.is_none() || atlas.height.is_none()) && (width > atlas.max_size || height > atlas.max_size) {
			fail!("could not pack images below MaxSize={} (last attempt {width}x{height})", atlas.max_size);
		}
		if let Some(positions) = try_pack(entries, width, height) {
			return Ok((width, height, positions));
		}
		match (atlas.width.is_some(), atlas.height.is_some()) {
			(true, true) => fail!("images do not fit fixed atlas size {width}x{height}"),
			(true, false) => height = height.checked_mul(2).ok_or_else(|| error!("atlas height overflow"))?,
			(false, true) => width = width.checked_mul(2).ok_or_else(|| error!("atlas width overflow"))?,
			(false, false) if width <= height => width = width.checked_mul(2).ok_or_else(|| error!("atlas width overflow"))?,
			(false, false) => height = height.checked_mul(2).ok_or_else(|| error!("atlas height overflow"))?,
		}
	}
}

fn try_pack(entries: &[Entry], width: i32, height: i32) -> Option<Vec<Point2i>> {
	let mut order = (0..entries.len()).collect::<Vec<_>>();
	order.sort_by(|&left, &right| {
		let (left_width, left_height) = entries[left].occupied_size();
		let (right_width, right_height) = entries[right].occupied_size();
		let left_size = (left_height, left_width);
		let right_size = (right_height, right_width);
		right_size.cmp(&left_size).then_with(|| entries[left].key.path.cmp(&entries[right].key.path))
	});
	let mut packer = image::GridBinPacker::new(width, height, 1, 1);
	let mut positions = vec![Point2i(0, 0); entries.len()];
	for index in order {
		let entry = &entries[index];
		let margin = entry.key.margin;
		let (width, height) = entry.occupied_size();
		let (x, y) = packer.insert(width, height)?;
		positions[index] = Point2i(x + margin, y + margin);
	}
	Some(positions)
}

fn render_preview(image: &image::ImageRGBA, kind: atlas::Kind, range: f32) -> image::ImageRGBA {
	if kind == atlas::Kind::Bitmap {
		return image.clone();
	}
	image.clone().map_colors(|pixel| {
		let distance = match kind {
			atlas::Kind::Sdf | atlas::Kind::Psdf => f32::from(pixel[0]) / 255.0,
			atlas::Kind::Msdf => {
				let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(|value| f32::from(value) / 255.0);
				r.min(g).max(r.max(g).min(b))
			},
			atlas::Kind::Mtsdf => f32::from(pixel[3]) / 255.0,
			atlas::Kind::Bitmap => unreachable!(),
		};
		let coverage = (range * (distance - 0.5) + 0.5).clamp(0.0, 1.0);
		let shade = (246.0 + (20.0 - 246.0) * coverage).round() as u8;
		[shade, shade, shade, 255]
	})
}

fn resolve_relative(base: &Path, value: &str) -> PathBuf {
	let path = Path::new(value);
	if path.is_absolute() { path.to_owned() } else { base.join(path) }
}

fn has_extension(path: &Path, extension: &str) -> bool {
	path.extension().and_then(OsStr::to_str).is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

fn next_power_of_two(value: i32) -> i32 {
	u32::try_from(value.max(1)).ok().and_then(u32::checked_next_power_of_two).and_then(|value| i32::try_from(value).ok()).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests;
