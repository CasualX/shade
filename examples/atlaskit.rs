use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, Command, value_parser};
use shade::atlas::{Atlas, Frame, Kind, Metadata, Sprite, Transform};
use shade::cvmath::{Point2i, Recti};
use shade::image::{BlitGutterMode, GridBinPacker, ImageRGBA};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
	let matches = Command::new("atlaskit")
		.about("Manipulate shade atlas JSON and textures")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommand(
			Command::new("addsprite")
				.about("Add a sprite image to an atlas")
				.arg(Arg::new("atlas_json").value_name("atlas.json").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("atlas_png").value_name("atlas.png").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("sprite").long("sprite").value_name("NAME").required(true))
				.arg(Arg::new("image").long("image").value_name("sprite.png").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("margin").long("margin").value_name("N").default_value("0").value_parser(value_parser!(i32)))
				.arg(Arg::new("transform").long("transform").value_name("TRANSFORM").default_value("None").value_parser(value_parser!(Transform)))
				.arg(Arg::new("origin").long("origin").value_name("X,Y").default_value("0,0").value_parser(value_parser!(Point2i))),
		)
		.subcommand(
			Command::new("fontatlas")
				.about("Convert msdf-atlas-gen font JSON into a shade atlas and merge an MTSDF sprite into the font texture")
				.arg(Arg::new("font_json").value_name("font.json").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("font_png").value_name("font.png").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("atlas_json").value_name("atlas.json").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("font").long("font").value_name("NAME").default_value("font"))
				.arg(Arg::new("sprite").long("sprite").value_name("NAME").required(true))
				.arg(Arg::new("image").long("image").value_name("sprite.png").required(true).value_parser(value_parser!(PathBuf)))
				.arg(Arg::new("margin").long("margin").value_name("N").default_value("0").value_parser(value_parser!(i32)))
				.arg(Arg::new("transform").long("transform").value_name("TRANSFORM").default_value("None").value_parser(value_parser!(Transform)))
				.arg(Arg::new("origin").long("origin").value_name("X,Y").default_value("0,0").value_parser(value_parser!(Point2i))),
		)
		.get_matches();

	match matches.subcommand() {
		Some(("addsprite", matches)) => {
			let atlas_json = matches.get_one::<PathBuf>("atlas_json").unwrap();
			let atlas_png = matches.get_one::<PathBuf>("atlas_png").unwrap();
			let sprite = matches.get_one::<String>("sprite").unwrap();
			let image = matches.get_one::<PathBuf>("image").unwrap();
			let margin = *matches.get_one::<i32>("margin").unwrap();
			let transform = *matches.get_one::<Transform>("transform").unwrap();
			let origin = *matches.get_one::<Point2i>("origin").unwrap();
			addsprite(atlas_json, atlas_png, sprite, image, margin, transform, origin)
		}
		Some(("fontatlas", matches)) => {
			let font_json = matches.get_one::<PathBuf>("font_json").unwrap();
			let font_png = matches.get_one::<PathBuf>("font_png").unwrap();
			let atlas_json = matches.get_one::<PathBuf>("atlas_json").unwrap();
			let font = matches.get_one::<String>("font").unwrap();
			let sprite = matches.get_one::<String>("sprite").unwrap();
			let image = matches.get_one::<PathBuf>("image").unwrap();
			let margin = *matches.get_one::<i32>("margin").unwrap();
			let transform = *matches.get_one::<Transform>("transform").unwrap();
			let origin = *matches.get_one::<Point2i>("origin").unwrap();
			fontatlas(font_json, font_png, atlas_json, font, sprite, image, margin, transform, origin)
		}
		_ => unreachable!("clap enforces subcommands"),
	}
}

fn addsprite(atlas_json_path: &Path, atlas_png_path: &Path, sprite_name: &str, image_path: &Path, margin: i32, transform: Transform, origin: Point2i) -> Result<()> {
	if margin < 0 {
		return Err("margin must be non-negative".into());
	}

	let mut atlas_image = ImageRGBA::load_file(atlas_png_path)?;
	let sprite_image = ImageRGBA::load_file(image_path)?;
	if sprite_image.width <= 0 || sprite_image.height <= 0 {
		return Err("sprite image must not be empty".into());
	}

	let mut atlas = load_atlas(atlas_json_path, atlas_image.width, atlas_image.height)?;
	if atlas.meta.width != atlas_image.width || atlas.meta.height != atlas_image.height {
		return Err(format!("atlas metadata is {}x{}, but texture is {}x{}", atlas.meta.width, atlas.meta.height, atlas_image.width, atlas_image.height).into());
	}
	if atlas.sprites.contains_key(sprite_name) {
		return Err(format!("sprite '{sprite_name}' already exists").into());
	}

	let mut packer = GridBinPacker::new(atlas.meta.width, atlas.meta.height, 1, 1);
	occupy_existing(&mut packer, &atlas)?;

	let occupied_width = sprite_image.width + margin * 2;
	let occupied_height = sprite_image.height + margin * 2;
	let Some((packed_x, packed_y)) = packer.insert(occupied_width, occupied_height) else {
		return Err(format!("no room for {}x{} sprite with margin {}", sprite_image.width, sprite_image.height, margin).into());
	};

	let rect = Recti {
		x: packed_x + margin,
		y: packed_y + margin,
		width: sprite_image.width,
		height: sprite_image.height,
	};
	atlas_image.copy_with_gutter(Point2i(rect.x, rect.y), &sprite_image, margin, BlitGutterMode::Edge);
	atlas.sprites.insert(sprite_name.to_owned(), Sprite::Frame(Frame {
		rect,
		margin,
		transform,
		origin,
	}));

	save_atlas(atlas_json_path, &atlas)?;
	atlas_image.save_file_png(atlas_png_path)?;
	println!("added sprite '{sprite_name}' at {},{} {}x{}", rect.x, rect.y, rect.width, rect.height);
	Ok(())
}

fn fontatlas(
	font_json_path: &Path,
	font_png_path: &Path,
	atlas_json_path: &Path,
	font_name: &str,
	sprite_name: &str,
	image_path: &Path,
	margin: i32,
	transform: Transform,
	origin: Point2i,
) -> Result<()> {
	if margin < 0 {
		return Err("margin must be non-negative".into());
	}

	let font_json = fs::read_to_string(font_json_path)?;
	let font_dto: shade::msdfgen::FontDto = serde_json::from_str(&font_json)?;
	let font: shade::atlas::Font = font_dto.into();
	let mut atlas_image = ImageRGBA::load_file(font_png_path)?;
	let sprite_image = ImageRGBA::load_file(image_path)?;
	if sprite_image.width <= 0 || sprite_image.height <= 0 {
		return Err("sprite image must not be empty".into());
	}
	if font.meta.width != atlas_image.width || font.meta.height != atlas_image.height {
		return Err(format!("font metadata is {}x{}, but texture is {}x{}", font.meta.width, font.meta.height, atlas_image.width, atlas_image.height).into());
	}

	let mut atlas = Atlas {
		version: 0,
		meta: font.meta,
		sprites: HashMap::new(),
		fonts: HashMap::from([(font_name.to_owned(), font)]),
	};

	let mut packer = GridBinPacker::new(atlas.meta.width, atlas.meta.height, 1, 1);
	occupy_existing(&mut packer, &atlas)?;

	let occupied_width = sprite_image.width + margin * 2;
	let occupied_height = sprite_image.height + margin * 2;
	let Some((packed_x, packed_y)) = packer.insert(occupied_width, occupied_height) else {
		return Err(format!("no room for {}x{} sprite with margin {}", sprite_image.width, sprite_image.height, margin).into());
	};

	let rect = Recti {
		x: packed_x + margin,
		y: packed_y + margin,
		width: sprite_image.width,
		height: sprite_image.height,
	};
	atlas_image.copy_with_gutter(Point2i(rect.x, rect.y), &sprite_image, margin, BlitGutterMode::Edge);
	atlas.sprites.insert(sprite_name.to_owned(), Sprite::Frame(Frame {
		rect,
		margin,
		transform,
		origin,
	}));

	save_atlas(atlas_json_path, &atlas)?;
	atlas_image.save_file_png(font_png_path)?;
	println!("wrote atlas '{}' with font '{}' and sprite '{}' at {},{} {}x{}", atlas_json_path.display(), font_name, sprite_name, rect.x, rect.y, rect.width, rect.height);
	Ok(())
}

fn load_atlas(path: &Path, width: i32, height: i32) -> Result<Atlas<String, String>> {
	if path.exists() {
		let data = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&data)?)
	}
	else {
		Ok(Atlas {
			version: 0,
			meta: Metadata {
				width,
				height,
				kind: Kind::Bitmap,
				distance_range: 0.0,
				distance_range_middle: 0.0,
			},
			sprites: HashMap::new(),
			fonts: HashMap::new(),
		})
	}
}

fn save_atlas(path: &Path, atlas: &Atlas<String, String>) -> Result<()> {
	let data = serde_json::to_string_pretty(atlas)?;
	fs::write(path, data)?;
	Ok(())
}

fn occupy_existing(packer: &mut GridBinPacker, atlas: &Atlas<String, String>) -> Result<()> {
	for (_name, sprite) in &atlas.sprites {
		for index in 0..sprite.len() {
			if let Some(frame) = sprite.get_frame(index) {
				occupy_frame(packer, frame);
			}
		}
	}
	for (_name, font) in &atlas.fonts {
		for glyph in &font.glyphs {
			if let Some(bounds) = &glyph.bounds {
				let frame = Frame {
					rect: bounds.atlas_bounds,
					margin: 0,
					transform: Transform::None,
					origin: Point2i(0, 0),
				};
				occupy_frame(packer, &frame);
			}
		}
	}
	Ok(())
}

fn occupy_frame(packer: &mut GridBinPacker, frame: &Frame) {
	let rect = Recti(frame.rect.x - frame.margin, frame.rect.y - frame.margin, frame.rect.width + frame.margin * 2, frame.rect.height + frame.margin * 2);
	packer.occupy(rect);
}
