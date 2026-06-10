use std::collections::HashSet;
use std::{fmt, mem};

use shade::cvmath::*;
use shade::{d2, d3};

pub mod examples;

pub trait DemoInterface {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::Continuous
	}

	fn resize(&mut self, _size: Vec2i) {}

	fn input(&mut self, _input: Input, _g: &mut shade::Graphics, _shell: &mut dyn ShellServices) {}

	fn file_opened(&mut self, _request_id: u32, _path: Option<String>, _bytes: Option<Vec<u8>>, _g: &mut shade::Graphics, _shell: &mut dyn ShellServices) {}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RedrawMode {
	OnDemand,
	Continuous,
}

#[derive(Copy, Clone, Debug)]
pub struct Frame {
	pub viewport: Bounds2i,
	pub time: f64,
	pub dt: f32,
}

pub trait ShellServices {
	fn request_redraw(&mut self) {}
	fn set_cursor(&mut self, _cursor: Cursor) {}
	fn set_pointer_capture(&mut self, _captured: bool) {}
	fn open_file(&mut self, _request: FileRequest) {}
	fn exit(&mut self) {}
	fn set_status(&mut self, _text: &str) {}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cursor {
	Default,
	Pointer,
	Grab,
	Grabbing,
	Crosshair,
	Move,
	ResizeEastWest,
	ResizeNorthSouth,
	ResizeNwse,
	ResizeNesw,
}

#[derive(Clone, Debug)]
pub struct FileRequest {
	pub id: u32,
	pub title: &'static str,
	pub extensions: &'static [&'static str],
}

#[derive(Clone, Debug)]
pub enum Input {
	MouseMove { position: Vec2f },
	MouseButton { button: MouseButton, pressed: bool, position: Vec2f },
	MouseWheel { delta: Vec2f, position: Vec2f },
	KeyDown(Key),
	KeyUp(Key),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Other(u16),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Key {
	Digit1,
	Digit2,
	Digit3,
	ArrowLeft,
	ArrowRight,
	ArrowUp,
	ArrowDown,
	F2,
	P,
	Shift,
	Escape,
	Other,
}

pub trait AssetLoader {
	fn read(&self, path: &str) -> Result<Vec<u8>, AssetError>;

	fn read_to_string(&self, path: &str) -> Result<String, AssetError> {
		let bytes = self.read(path)?;
		String::from_utf8(bytes).map_err(|source| AssetError {
			path: path.to_owned(),
			message: source.to_string(),
		})
	}
}

#[derive(Clone, Debug)]
pub struct AssetError {
	pub path: String,
	pub message: String,
}

impl AssetError {
	pub fn missing(path: &str) -> AssetError {
		AssetError {
			path: path.to_owned(),
			message: "asset not found".to_owned(),
		}
	}
}

impl fmt::Display for AssetError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.path, self.message)
	}
}

impl std::error::Error for AssetError {}

pub fn load_font(g: &mut shade::Graphics, assets: &dyn AssetLoader, text_3d: bool) -> shade::d2::FontResource<shade::msdfgen::Font> {
	// Load the font metadata
	let font = {
		let text = assets.read_to_string("font/font.json").expect("failed to load font metadata");
		let font: shade::msdfgen::FontDto = serde_json::from_str(&text).expect("failed to parse font metadata");
		font.into()
	};
	// Load the font texture
	let texture = {
		let data = assets.read("font/font.png").expect("failed to load font texture");
		let image = shade::image::ImageRGBA::load_memory_png(&data).expect("failed to decode font texture");
		let image = image.map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
		let props = shade::TextureProps {
			mip_levels: 1,
			usage: shade::TextureUsage::TEXTURE,
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Edge,
			wrap_v: shade::TextureWrap::Edge,
			..Default::default()
		};

		/*
		shade::TextureProps! {
			mip_levels: 1,
			usage: TEXTURE,
			filter: Linear,
			wrap: Edge,
		}
		
		 */
		g.image(&props.bind(&image))
	};
	// Load the font shader

	let mut shader_source = shade::shader_interface! {
		files {
			"mtsdf.glsl" => shade::shaders::MTSDF,
		}
	};
	let defines: &[_] = if text_3d { &[shade::ShaderDefine { name: "MTSDF_3D", value: None }] } else { &[] };
	let shader = g.shader_compile(&mut shader_source, "mtsdf.glsl", defines);
	shade::d2::FontResource { font, texture, shader }
}
