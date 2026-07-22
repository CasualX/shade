use std::{error, fmt};

use shade::cvmath::*;
use shade::{atlas, msdfgen};

#[derive(Debug)]
pub struct Error {
	line: usize,
	message: String,
}

impl Error {
	fn document<T>(line: usize, message: impl fmt::Display) -> Result<T, Error> {
		Err(Error { line, message: message.to_string() })
	}

	fn value(line: usize, key: &str, value: &str, expected: &str) -> Error {
		Error {
			line,
			message: format!("invalid {key} value '{value}'; expected {expected}"),
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.message)
	}
}

impl error::Error for Error {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum SpriteProcessor {
	Blit,
	Msdfgen,
}

impl SpriteProcessor {
	fn parse(value: &str) -> Option<SpriteProcessor> {
		match value {
			"blit" => Some(SpriteProcessor::Blit),
			"msdfgen" => Some(SpriteProcessor::Msdfgen),
			_ => None,
		}
	}
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum GutterMode {
	#[default]
	ClampToEdge,
	SelfTiled,
	Transparent,
}

impl GutterMode {
	fn parse(value: &str) -> Option<GutterMode> {
		match value {
			"ClampToEdge" => Some(GutterMode::ClampToEdge),
			"SelfTiled" => Some(GutterMode::SelfTiled),
			"Transparent" => Some(GutterMode::Transparent),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Manifest {
	pub kind: Option<atlas::Kind>,
	pub distance_range: Option<f32>,
	pub margin: Option<i32>,
	pub width: Option<i32>,
	pub height: Option<i32>,
	pub max_size: Option<i32>,
	pub gutter: Option<GutterMode>,
	pub recover_alpha_colors: Option<bool>,
	pub sprite_processor: Option<SpriteProcessor>,
	pub msdfgen: Msdfgen,
	pub msdf_atlas_gen: MsdfAtlasGen,
	pub sprites: Vec<Sprite>,
	pub fonts: Vec<Font>,
}

impl Manifest {
	fn set_property(&mut self, line: usize, key: &str, value: &str) -> Result<(), Error> {
		if self.msdfgen.set_property(line, key, value)? {
			return Ok(());
		}
		if self.msdf_atlas_gen.set_property(line, key, value)? {
			return Ok(());
		}
		match key {
			"Kind" => set_once(line, key, &mut self.kind, value.parse().map_err(|_| Error::value(line, key, value, "bitmap, sdf, psdf, msdf, or mtsdf"))?),
			"DistanceRange" => set_once(line, key, &mut self.distance_range, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Msdfgen.Mode" => set_once(line, key, &mut self.msdfgen.mode, value.parse().map_err(|_| Error::value(line, key, value, "sdf, psdf, msdf, or mtsdf"))?),
			"Msdfgen.Range" => set_once(line, key, &mut self.msdfgen.range, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"MsdfAtlasGen.Mode" => set_once(line, key, &mut self.msdf_atlas_gen.mode, value.parse().map_err(|_| Error::value(line, key, value, "sdf, psdf, msdf, or mtsdf"))?),
			"MsdfAtlasGen.Range" => set_once(line, key, &mut self.msdf_atlas_gen.range, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Margin" => set_once(line, key, &mut self.margin, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Width" => set_once(line, key, &mut self.width, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Height" => set_once(line, key, &mut self.height, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"MaxSize" => set_once(line, key, &mut self.max_size, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Blit.Gutter" => set_once(line, key, &mut self.gutter, GutterMode::parse(value).ok_or_else(|| Error::value(line, key, value, "ClampToEdge, SelfTiled, or Transparent"))?),
			"RecoverAlphaColors" => set_once(line, key, &mut self.recover_alpha_colors, value.parse().map_err(|_| Error::value(line, key, value, "true or false"))?),
			"Processor" => set_once(line, key, &mut self.sprite_processor, SpriteProcessor::parse(value).ok_or_else(|| Error::value(line, key, value, "blit or msdfgen"))?),
			_ => Error::document(line, format!("unknown manifest property '{key}'")),
		}
	}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Msdfgen {
	pub mode: Option<msdfgen::Mode>,
	pub range: Option<f32>,
	pub size: Option<f64>,
	pub autoframe: Option<bool>,
	pub y_flip: Option<bool>,
	pub overlap: Option<bool>,
	pub fill_rule: Option<msdfgen::FillRule>,
}

impl Msdfgen {
	fn set_property(&mut self, line: usize, key: &str, value: &str) -> Result<bool, Error> {
		match key {
			"Msdfgen.Size" => set_once(line, key, &mut self.size, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Msdfgen.Autoframe" => set_once(line, key, &mut self.autoframe, value.parse().map_err(|_| Error::value(line, key, value, "true or false"))?),
			"Msdfgen.YFlip" => set_once(line, key, &mut self.y_flip, value.parse().map_err(|_| Error::value(line, key, value, "true or false"))?),
			"Msdfgen.Overlap" => set_once(line, key, &mut self.overlap, value.parse().map_err(|_| Error::value(line, key, value, "true or false"))?),
			"Msdfgen.FillRule" => set_once(line, key, &mut self.fill_rule, value.parse().map_err(|_| Error::value(line, key, value, "nonzero, evenodd, positive, or negative"))?),
			_ => return Ok(false),
		}?;
		Ok(true)
	}
}

#[derive(Clone, Debug, Default)]
pub struct MsdfAtlasGen {
	pub mode: Option<msdfgen::Mode>,
	pub range: Option<f32>,
	pub size: Option<f64>,
	pub outer_padding: Option<f64>,
	pub overlap: Option<bool>,
	pub charset: Option<String>,
	pub chars: Option<String>,
}

impl MsdfAtlasGen {
	fn set_property(&mut self, line: usize, key: &str, value: &str) -> Result<bool, Error> {
		match key {
			"MsdfAtlasGen.Size" => set_once(line, key, &mut self.size, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"MsdfAtlasGen.OuterPadding" => set_once(line, key, &mut self.outer_padding, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"MsdfAtlasGen.Overlap" => set_once(line, key, &mut self.overlap, value.parse().map_err(|_| Error::value(line, key, value, "true or false"))?),
			_ => return Ok(false),
		}?;
		Ok(true)
	}

	fn set_input_property(&mut self, line: usize, key: &str, value: &str) -> Result<bool, Error> {
		match key {
			"MsdfAtlasGen.Charset" => set_once(line, key, &mut self.charset, value.to_owned()),
			"MsdfAtlasGen.Chars" => set_once(line, key, &mut self.chars, value.to_owned()),
			_ => return Ok(false),
		}?;
		Ok(true)
	}
}

#[derive(Clone, Debug, Default)]
pub struct Sprite {
	pub name: String,
	pub path: Option<String>,
	pub duration: Option<f32>,
	pub processor: Option<SpriteProcessor>,
	pub margin: Option<i32>,
	pub gutter: Option<GutterMode>,
	pub transform: Option<atlas::Transform>,
	pub origin: Option<Point2i>,
	pub msdfgen: Msdfgen,
}

impl Sprite {
	fn set_property(&mut self, line: usize, key: &str, value: &str) -> Result<(), Error> {
		if self.msdfgen.set_property(line, key, value)? {
			return Ok(());
		}
		match key {
			"Path" => set_once(line, key, &mut self.path, value.to_owned()),
			"Duration" => set_once(line, key, &mut self.duration, parse_duration(value).ok_or_else(|| Error::value(line, key, value, "a number or X/Y"))?),
			"Processor" => set_once(line, key, &mut self.processor, SpriteProcessor::parse(value).ok_or_else(|| Error::value(line, key, value, "blit or msdfgen"))?),
			"Margin" => set_once(line, key, &mut self.margin, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Blit.Gutter" => set_once(line, key, &mut self.gutter, GutterMode::parse(value).ok_or_else(|| Error::value(line, key, value, "ClampToEdge, SelfTiled, or Transparent"))?),
			"Transform" => set_once(line, key, &mut self.transform, value.parse().map_err(|_| Error::value(line, key, value, "a Shade Transform"))?),
			"Origin" => set_once(line, key, &mut self.origin, value.parse().map_err(|_| Error::value(line, key, value, "X,Y"))?),
			_ => Error::document(line, format!("unknown sprite property '{key}'")),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Font {
	pub name: String,
	pub path: Option<String>,
	pub font_scale: Option<f64>,
	pub margin: Option<i32>,
	pub msdf_atlas_gen: MsdfAtlasGen,
}

impl Font {
	fn set_property(&mut self, line: usize, key: &str, value: &str) -> Result<(), Error> {
		if self.msdf_atlas_gen.set_property(line, key, value)? {
			return Ok(());
		}
		if self.msdf_atlas_gen.set_input_property(line, key, value)? {
			return Ok(());
		}
		match key {
			"Path" => set_once(line, key, &mut self.path, value.to_owned()),
			"FontScale" => set_once(line, key, &mut self.font_scale, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			"Margin" => set_once(line, key, &mut self.margin, value.parse().map_err(|_| Error::value(line, key, value, "a number"))?),
			_ => Error::document(line, format!("unknown font property '{key}'")),
		}
	}
}

impl Manifest {
	pub fn parse(text: &str) -> Result<Manifest, Error> {
		#[derive(Copy, Clone)]
		enum Section {
			Sprite(usize),
			Font(usize),
		}

		let mut manifest = Manifest::default();
		let mut current_section = None;
		let mut parser = ini_core::Parser::new(text).auto_trim(true).comment_char(b'#');

		loop {
			let line = parser.line() as usize + 1;
			let Some(item) = parser.next() else { break };
			match item {
				ini_core::Item::Blank | ini_core::Item::Comment(_) | ini_core::Item::SectionEnd => {},
				ini_core::Item::Error(_) => return Error::document(line, "unterminated section header"),
				ini_core::Item::Section(section) => {
					let Some((kind, name)) = section.split_once(':') else {
						return Error::document(line, "section must use the [Sprite:name] or [Font:name] form");
					};
					if name.is_empty() {
						return Error::document(line, "resource name must not be empty");
					}
					match kind {
						"Sprite" => {
							manifest.sprites.push(Sprite { name: name.to_owned(), ..Sprite::default() });
							current_section = Some(Section::Sprite(manifest.sprites.len() - 1));
						},
						"Font" => {
							manifest.fonts.push(Font { name: name.to_owned(), ..Font::default() });
							current_section = Some(Section::Font(manifest.fonts.len() - 1));
						},
						_ => return Error::document(line, format!("unsupported resource section '{kind}'")),
					}
				},
				ini_core::Item::Property(_, None) => return Error::document(line, "expected Key=Value"),
				ini_core::Item::Property(key, Some(value)) => {
					match current_section {
						Some(Section::Sprite(index)) => manifest.sprites[index].set_property(line, key, value)?,
						Some(Section::Font(index)) => manifest.fonts[index].set_property(line, key, value)?,
						None => manifest.set_property(line, key, value)?,
					}
				},
			}
		}

		Ok(manifest)
	}
}

fn set_once<T>(line: usize, key: &str, slot: &mut Option<T>, value: T) -> Result<(), Error> {
	if slot.is_some() {
		return Error::document(line, format!("duplicate property '{key}'"));
	}
	*slot = Some(value);
	Ok(())
}

fn parse_duration(value: &str) -> Option<f32> {
	match value.split_once('/') {
		Some((numerator, denominator)) => Some(numerator.trim().parse::<f32>().ok()? / denominator.trim().parse::<f32>().ok()?),
		None => value.parse().ok(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_document_dom() {
		let document = Manifest::parse("Kind=msdf\nDistanceRange=6\nMargin=2\nBlit.Gutter=SelfTiled\nRecoverAlphaColors=true\nMsdfgen.Size=32\nMsdfAtlasGen.Size=36\n[Sprite:icon]\nPath=icon.svg\nBlit.Gutter=Transparent\nOrigin=-2, 3\n[Font:UI]\nPath=ui.ttf\nMsdfAtlasGen.Charset=ui.txt\nMsdfAtlasGen.Size=40\n").unwrap();
		assert_eq!(document.kind, Some(atlas::Kind::Msdf));
		assert_eq!(document.distance_range, Some(6.0));
		assert_eq!(document.margin, Some(2));
		assert_eq!(document.gutter, Some(GutterMode::SelfTiled));
		assert_eq!(document.recover_alpha_colors, Some(true));
		assert_eq!(document.msdfgen.size, Some(32.0));
		assert_eq!(document.msdf_atlas_gen.size, Some(36.0));
		assert_eq!(document.sprites.len(), 1);
		assert_eq!(document.sprites[0].name, "icon");
		assert_eq!(document.sprites[0].path.as_deref(), Some("icon.svg"));
		assert_eq!(document.sprites[0].gutter, Some(GutterMode::Transparent));
		assert_eq!(document.sprites[0].origin, Some(Point2i(-2, 3)));
		assert_eq!(document.fonts.len(), 1);
		assert_eq!(document.fonts[0].name, "UI");
		assert_eq!(document.fonts[0].path.as_deref(), Some("ui.ttf"));
		assert_eq!(document.fonts[0].msdf_atlas_gen.charset.as_deref(), Some("ui.txt"));
		assert_eq!(document.fonts[0].msdf_atlas_gen.size, Some(40.0));
	}

	#[test]
	fn parse_errors_retain_line_context() {
		let error = Manifest::parse("Margin=2\nWidth=wide").unwrap_err();
		assert_eq!(error.line, 2);
		assert_eq!(error.message, "invalid Width value 'wide'; expected a number");
		assert_eq!(error.to_string(), "line 2: invalid Width value 'wide'; expected a number");
	}

	#[test]
	fn rejects_execution_options() {
		assert!(Manifest::parse("TempDir=tmp").is_err());
		assert!(Manifest::parse("KeepIntermediate=true").is_err());
	}

	#[test]
	fn values_are_case_sensitive() {
		assert!(Manifest::parse("Kind=MSDF").is_err());
		assert!(Manifest::parse("Blit.Gutter=selftiled").is_err());
		assert!(Manifest::parse("Blit.Gutter=Unknown").is_err());
		assert!(Manifest::parse("Msdfgen.Autoframe=True").is_err());
		assert!(Manifest::parse("Msdfgen.FillRule=EvenOdd").is_err());
		assert!(Manifest::parse("RecoverAlphaColors=True").is_err());
		assert!(Manifest::parse("Margin=1\nMargin=2").is_err());
	}

	#[test]
	fn parses_root_msdfgen_settings_without_bitmap() {
		let document = Manifest::parse("Msdfgen.Mode=mtsdf\nMsdfgen.Range=6").unwrap();
		assert_eq!(document.msdfgen.mode, Some(msdfgen::Mode::Mtsdf));
		assert_eq!(document.msdfgen.range, Some(6.0));
		assert!(Manifest::parse("Msdfgen.Mode=bitmap").is_err());
	}

	#[test]
	fn parses_msdf_atlas_gen_settings_separately() {
		let document = Manifest::parse("Kind=psdf\nDistanceRange=9\nMsdfgen.Mode=msdf\nMsdfgen.Range=3\nMsdfgen.Size=24\nMsdfgen.Overlap=true\nMsdfAtlasGen.Mode=mtsdf\nMsdfAtlasGen.Range=6\nMsdfAtlasGen.Size=48\nMsdfAtlasGen.OuterPadding=2\nMsdfAtlasGen.Overlap=false").unwrap();
		assert_eq!(document.kind, Some(atlas::Kind::Psdf));
		assert_eq!(document.distance_range, Some(9.0));
		assert_eq!(document.msdfgen.mode, Some(msdfgen::Mode::Msdf));
		assert_eq!(document.msdfgen.range, Some(3.0));
		assert_eq!(document.msdfgen.size, Some(24.0));
		assert_eq!(document.msdfgen.overlap, Some(true));
		assert_eq!(document.msdf_atlas_gen.mode, Some(msdfgen::Mode::Mtsdf));
		assert_eq!(document.msdf_atlas_gen.range, Some(6.0));
		assert_eq!(document.msdf_atlas_gen.size, Some(48.0));
		assert_eq!(document.msdf_atlas_gen.outer_padding, Some(2.0));
		assert_eq!(document.msdf_atlas_gen.overlap, Some(false));
	}

	#[test]
	fn rejects_sprite_mode_and_range_overrides() {
		assert!(Manifest::parse("[Sprite:Icon]\nMsdfgen.Mode=msdf").is_err());
		assert!(Manifest::parse("[Sprite:Icon]\nMsdfgen.Range=6").is_err());
	}

	#[test]
	fn rejects_unprefixed_msdf_atlas_gen_font_settings() {
		assert!(Manifest::parse("[Font:Ui]\nSize=32").is_err());
		assert!(Manifest::parse("[Font:Ui]\nOuterPadding=1").is_err());
		assert!(Manifest::parse("[Font:Ui]\nMsdfgen.Overlap=true").is_err());
		assert!(Manifest::parse("[Font:Ui]\nCharset=ascii.txt").is_err());
		assert!(Manifest::parse("[Font:Ui]\nChars=abc").is_err());
		assert!(Manifest::parse("[Font:Ui]\nAllGlyphs=true").is_err());
	}

	#[test]
	fn keeps_msdf_atlas_gen_input_settings_in_font_sections() {
		let document = Manifest::parse("[Font:Ui]\nMsdfAtlasGen.Chars=abc\nFontScale=0.75").unwrap();
		assert_eq!(document.fonts[0].msdf_atlas_gen.chars.as_deref(), Some("abc"));
		assert_eq!(document.fonts[0].font_scale, Some(0.75));
		assert!(Manifest::parse("MsdfAtlasGen.Charset=ascii.txt").is_err());
		assert!(Manifest::parse("MsdfAtlasGen.Chars=abc").is_err());
		assert!(Manifest::parse("[Font:Ui]\nMsdfAtlasGen.AllGlyphs=true").is_err());
	}

	#[test]
	fn repeated_font_names_create_variants() {
		let document = Manifest::parse("[Font:UI]\nPath=regular.ttf\n[Font:UI]\nPath=symbols.ttf\nMsdfAtlasGen.Chars=\"★\"\n").unwrap();
		assert_eq!(document.fonts.len(), 2);
		assert_eq!(document.fonts[0].name, "UI");
		assert_eq!(document.fonts[1].name, "UI");
	}

	#[test]
	fn repeated_sprite_names_create_animation_frames() {
		let document = Manifest::parse("[Sprite:Coin]\nPath=coin-1.png\nDuration=2/60\n[Sprite:Coin]\nPath=coin-2.png\nDuration=0.2\n").unwrap();
		assert_eq!(document.sprites.len(), 2);
		assert_eq!(document.sprites[0].name, "Coin");
		assert_eq!(document.sprites[0].duration, Some(2.0 / 60.0));
		assert_eq!(document.sprites[1].name, "Coin");
		assert_eq!(document.sprites[1].duration, Some(0.2));
	}

	#[test]
	fn duration_ratios_require_two_floats() {
		assert_eq!(parse_duration("2 / 60"), Some(2.0 / 60.0));
		assert_eq!(parse_duration("1.5/0.5"), Some(3.0));
		assert_eq!(parse_duration("0.25"), Some(0.25));
		assert_eq!(parse_duration("2/frames"), None);
		assert_eq!(parse_duration("2/60/2"), None);
		assert_eq!(parse_duration("/60"), None);
	}
}
