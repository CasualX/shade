use super::*;

#[test]
fn infers_processor_from_extension() {
	assert_eq!(resolve_sprite_processor(None, Path::new("icon.svg")), manifest::SpriteProcessor::Msdfgen);
	assert_eq!(resolve_sprite_processor(None, Path::new("icon.SVG")), manifest::SpriteProcessor::Msdfgen);
	assert_eq!(resolve_sprite_processor(None, Path::new("icon.png")), manifest::SpriteProcessor::Blit);
	assert_eq!(resolve_sprite_processor(Some(manifest::SpriteProcessor::Blit), Path::new("icon.svg")), manifest::SpriteProcessor::Blit);
}

#[test]
fn parses_svg_view_box_attribute() {
	let svg = r#"<!-- viewBox="9 9 9 9" --><svg viewBox="-1, 2 32 24"></svg>"#;
	let view_box = parse_view_box_source(Path::new("test.svg"), svg).unwrap();
	assert_eq!((view_box.x, view_box.y, view_box.width, view_box.height), (-1.0, 2.0, 32.0, 24.0));
}

#[test]
fn rejects_nested_svg_element() {
	let svg = r#"<div><svg viewBox="0 0 32 32"></svg></div>"#;
	assert!(parse_view_box_source(Path::new("test.svg"), svg).is_err());
}

#[test]
fn deduplicates_sprite_pixels_independently_of_export_metadata() {
	let processor = ProcessorSpec::Msdfgen(MsdfgenSpec {
		mode: msdfgen::Mode::Mtsdf,
		range: 4.0,
		size: 32.0,
		autoframe: false,
		y_flip: false,
		overlap: false,
		fill_rule: msdfgen::FillRule::NonZero,
	});
	let sprites = vec![
		SpriteSpec {
			name: "Up".to_owned(),
			path: PathBuf::from("arrow.svg"),
			duration: None,
			processor: processor.clone(),
			margin: 2,
			gutter: manifest::GutterMode::ClampToEdge,
			transform: atlas::Transform::None,
			origin: Point2i(0, 0),
		},
		SpriteSpec {
			name: "Down".to_owned(),
			path: PathBuf::from("arrow.svg"),
			duration: None,
			processor,
			margin: 2,
			gutter: manifest::GutterMode::Transparent,
			transform: atlas::Transform::Rotate180,
			origin: Point2i(4, 8),
		},
	];

	let (jobs, sprite_entry_indices) = plan_loads(&sprites);
	assert_eq!(jobs.len(), 1);
	assert_eq!(sprite_entry_indices, [0, 0]);
}

#[test]
fn bitmap_gutter_mode_participates_in_pixel_deduplication() {
	let sprites = vec![
		SpriteSpec {
			name: "Edge".to_owned(),
			path: PathBuf::from("tile.png"),
			duration: None,
			processor: ProcessorSpec::Blit,
			margin: 1,
			gutter: manifest::GutterMode::ClampToEdge,
			transform: atlas::Transform::None,
			origin: Point2i(0, 0),
		},
		SpriteSpec {
			name: "Tiled".to_owned(),
			path: PathBuf::from("tile.png"),
			duration: None,
			processor: ProcessorSpec::Blit,
			margin: 1,
			gutter: manifest::GutterMode::SelfTiled,
			transform: atlas::Transform::None,
			origin: Point2i(0, 0),
		},
	];

	let (jobs, sprite_entry_indices) = plan_loads(&sprites);
	assert_eq!(jobs.len(), 2);
	assert_eq!(sprite_entry_indices, [0, 1]);
}

#[test]
fn bitmap_gutter_modes_select_the_expected_blit_behavior() {
	let left = [10, 20, 30, 255];
	let right = [40, 50, 60, 255];
	let source = image::ImageRGBA::from_raw(2, 1, vec![left, right]);
	let render = |mode| {
		let mut output = image::ImageRGBA::new(4, 3, [0, 0, 0, 0]);
		output.copy_with_gutter(Point2i(1, 1), &source, 1, bitmap_gutter_mode(mode));
		output
	};

	let edge = render(manifest::GutterMode::ClampToEdge);
	assert_eq!(edge.read(0, 0), Some(left));
	assert_eq!(edge.read(3, 2), Some(right));

	let tiled = render(manifest::GutterMode::SelfTiled);
	assert_eq!(tiled.read(0, 0), Some(right));
	assert_eq!(tiled.read(3, 2), Some(left));

	let transparent = render(manifest::GutterMode::Transparent);
	assert_eq!(transparent.read(0, 0), Some([0, 0, 0, 0]));
	assert_eq!(transparent.read(1, 1), Some(left));
	assert_eq!(transparent.read(2, 1), Some(right));
}

#[test]
fn alpha_color_recovery_is_opt_in() {
	let transparent = [0, 0, 0, 0];
	let image = image::ImageRGBA::from_raw(3, 1, vec![[200, 100, 50, 255], transparent, [100, 50, 200, 255]]);

	assert_eq!(apply_alpha_color_recovery(image.clone(), false).read(1, 0), Some(transparent));
	assert_eq!(apply_alpha_color_recovery(image, true).read(1, 0), Some([150, 75, 125, 0]));
}

#[test]
fn atlas_metadata_is_independent_of_font_generator_settings() {
	let spec = AtlasSpec {
		kind: atlas::Kind::Bitmap,
		distance_range: 0.0,
		width: None,
		height: None,
		max_size: 4096,
		recover_alpha_colors: false,
		sprites: Vec::new(),
		fonts: vec![FontSpec {
			name: "ui".to_owned(),
			inputs: Vec::new(),
			margin: 2,
			msdf_atlas_gen: MsdfAtlasGenSpec {
				mode: msdfgen::Mode::Msdf,
				range: 6.0,
				size: 32.0,
				outer_padding: 1.0,
				overlap: false,
			},
		}],
		msdf_atlas_gen: MsdfAtlasGenSpec {
			mode: msdfgen::Mode::Msdf,
			range: 6.0,
			size: 32.0,
			outer_padding: 1.0,
			overlap: false,
		},
	};
	assert_eq!((spec.kind, spec.distance_range), (atlas::Kind::Bitmap, 0.0));
	assert!(!spec.recover_alpha_colors);
	assert_eq!((spec.fonts[0].msdf_atlas_gen.mode, spec.fonts[0].msdf_atlas_gen.range), (msdfgen::Mode::Msdf, 6.0));
}

#[test]
fn detects_msdfgen_metadata_mismatches() {
	assert_eq!(msdfgen_metadata_mismatches(atlas::Kind::Mtsdf, 4.0, msdfgen::Mode::Mtsdf, 4.0), (false, false));
	assert_eq!(msdfgen_metadata_mismatches(atlas::Kind::Msdf, 6.0, msdfgen::Mode::Mtsdf, 4.0), (true, true));
}

#[test]
fn fixed_atlas_dimensions_ignore_max_size() {
	let mut spec = AtlasSpec {
		kind: atlas::Kind::Bitmap,
		distance_range: 0.0,
		width: Some(64),
		height: Some(32),
		max_size: 0,
		recover_alpha_colors: false,
		sprites: Vec::new(),
		fonts: Vec::new(),
		msdf_atlas_gen: MsdfAtlasGenSpec {
			mode: msdfgen::Mode::Mtsdf,
			range: 4.0,
			size: 32.0,
			outer_padding: 1.0,
			overlap: false,
		},
	};

	assert!(validate_defaults(Path::new("atlas.ini"), &spec, 0, 4.0).is_ok());
	let (width, height, positions) = pack_entries(&spec, &[]).unwrap();
	assert_eq!((width, height, positions), (64, 32, Vec::new()));

	spec.height = None;
	assert!(validate_defaults(Path::new("atlas.ini"), &spec, 0, 4.0).is_err());
	assert!(pack_entries(&spec, &[]).is_err());
}

#[test]
fn validates_sprite_durations_against_name_repetition() {
	let parse = |source| manifest::Manifest::parse(source).unwrap().sprites;
	let path = Path::new("atlas.ini");

	assert!(validate_sprite_durations(path, &parse("[Sprite:Coin]\nPath=coin.png\n")).is_ok());
	assert!(validate_sprite_durations(path, &parse("[Sprite:Coin]\nPath=coin.png\nDuration=0.1\n")).is_err());
	assert!(validate_sprite_durations(path, &parse("[Sprite:Coin]\nPath=a.png\nDuration=0.1\n[Sprite:Coin]\nPath=b.png\nDuration=0.2\n")).is_ok());
	assert!(validate_sprite_durations(path, &parse("[Sprite:Coin]\nPath=a.png\nDuration=0.1\n[Sprite:Coin]\nPath=b.png\n")).is_err());
}

#[test]
fn repeated_sprite_specs_emit_an_animated_sprite_in_manifest_order() {
	let sprite = |name: &str, duration, transform, origin| SpriteSpec {
		name: name.to_owned(),
		path: PathBuf::from("sprite.png"),
		duration,
		processor: ProcessorSpec::Blit,
		margin: 1,
		gutter: manifest::GutterMode::ClampToEdge,
		transform,
		origin,
	};
	let specs = vec![
		sprite("Coin", Some(0.1), atlas::Transform::None, Point2i(1, 2)),
		sprite("Coin", Some(0.2), atlas::Transform::FlipX, Point2i(3, 4)),
		sprite("Player", None, atlas::Transform::None, Point2i(5, 6)),
	];
	let frames = vec![
		atlas::Frame { rect: Recti(10, 0, 8, 8), ..atlas::Frame::default() },
		atlas::Frame { rect: Recti(20, 0, 8, 8), ..atlas::Frame::default() },
		atlas::Frame { rect: Recti(30, 0, 8, 8), ..atlas::Frame::default() },
	];
	let sprites = build_sprites(&specs, &[1, 0, 2], &frames);

	let atlas::Sprite::Animated(coin) = &sprites["Coin"] else { panic!("Coin must be animated") };
	assert_eq!(coin.len(), 2);
	assert_eq!((coin[0].frame.rect, coin[0].duration, coin[0].frame.origin), (frames[1].rect, 0.1, Point2i(1, 2)));
	assert_eq!((coin[1].frame.rect, coin[1].duration, coin[1].frame.transform), (frames[0].rect, 0.2, atlas::Transform::FlipX));
	let atlas::Sprite::Frame(player) = &sprites["Player"] else { panic!("Player must be static") };
	assert_eq!((player.rect, player.origin), (frames[2].rect, Point2i(5, 6)));
}

#[test]
fn rejects_autoframe_with_a_nonzero_margin() {
	let mut sprite = SpriteSpec {
		name: "Icon".to_owned(),
		path: PathBuf::from("icon.svg"),
		duration: None,
		processor: ProcessorSpec::Msdfgen(MsdfgenSpec {
			mode: msdfgen::Mode::Mtsdf,
			range: 4.0,
			size: 32.0,
			autoframe: true,
			y_flip: false,
			overlap: false,
			fill_rule: msdfgen::FillRule::NonZero,
		}),
		margin: 2,
		gutter: manifest::GutterMode::ClampToEdge,
		transform: atlas::Transform::None,
		origin: Point2i(0, 0),
	};
	assert!(validate_sprite(Path::new("atlas.ini"), &sprite).is_err());
	sprite.margin = 0;
	assert!(validate_sprite(Path::new("atlas.ini"), &sprite).is_ok());
}

#[test]
fn relocates_font_glyphs_into_final_atlas() {
	let font = atlas::Font {
		meta: atlas::Metadata {
			width: 64,
			height: 32,
			kind: atlas::Kind::Mtsdf,
			distance_range: 4.0,
			distance_range_middle: 0.0,
		},
		metrics: Vec::new(),
		glyphs: vec![atlas::Glyph {
			metrics_index: 0,
			advance: 1.0,
			bounds: Some(atlas::GlyphBounds {
				plane_bounds: atlas::PlaneBounds::default(),
				atlas_bounds: Recti(3, 4, 5, 6),
			}),
		}],
		codepoints: HashMap::new(),
		kerning: HashMap::new(),
	};
	let font = relocate_font(font, &[Some(1)], &[Point2i(100, 100), Point2i(10, 20)], (256, 128));
	assert_eq!((font.meta.width, font.meta.height), (256, 128));
	assert_eq!(font.glyphs[0].bounds.unwrap().atlas_bounds, Recti(10, 20, 5, 6));
}
