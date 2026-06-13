pub mod api;

use shade::cvmath::*;
use shade::gui;
use demos::*;

#[cfg(target_family = "wasm")]
use core::num::NonZeroU32;
#[cfg(target_family = "wasm")]
use getrandom::{Error as GetRandomError, register_custom_getrandom};

#[cfg(target_family = "wasm")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
	#[link_name = "setCursor"]
	fn set_cursor_import(cursor: u32);
	#[link_name = "openFile"]
	fn open_file_import(request_id: u32, title_ptr: *const u8, title_len: usize, extensions_ptr: *const u8, extensions_len: usize);
	#[link_name = "setStatus"]
	fn set_status_import(text_ptr: *const u8, text_len: usize);
	#[link_name = "randomFill"]
	fn random_fill_import(dest: *mut u8, len: usize) -> bool;
}

#[cfg(not(target_family = "wasm"))]
unsafe fn set_cursor_import(_cursor: u32) {}

#[cfg(not(target_family = "wasm"))]
unsafe fn open_file_import(_request_id: u32, _title_ptr: *const u8, _title_len: usize, _extensions_ptr: *const u8, _extensions_len: usize) {}

#[cfg(not(target_family = "wasm"))]
unsafe fn set_status_import(_text_ptr: *const u8, _text_len: usize) {}

#[cfg(target_family = "wasm")]
const WEB_RANDOM_ERROR_CODE: u32 = GetRandomError::CUSTOM_START + 1;

#[cfg(target_family = "wasm")]
fn web_getrandom(dest: &mut [u8]) -> Result<(), GetRandomError> {
	let ok = unsafe { random_fill_import(dest.as_mut_ptr(), dest.len()) };
	if ok {
		Ok(())
	}
	else {
		Err(GetRandomError::from(NonZeroU32::new(WEB_RANDOM_ERROR_CODE).unwrap()))
	}
}

#[cfg(target_family = "wasm")]
register_custom_getrandom!(web_getrandom);

pub trait DemoContext {
	fn resize(&mut self, width: i32, height: i32);
	fn draw(&mut self, time: f64);
	fn redraw_mode(&self) -> RedrawMode;
	fn take_redraw_request(&mut self) -> bool;
	fn file_opened(&mut self, request_id: u32, path: Option<String>, bytes: Option<Vec<u8>>);

	fn mousemove(&mut self, _dx: f32, _dy: f32) {}
	fn mousedown(&mut self, _button: u32) {}
	fn mouseup(&mut self, _button: u32) {}
	fn wheel(&mut self, _delta_y: f32) {}
	fn keydown(&mut self, _key: u32) {}
	fn keyup(&mut self, _key: u32) {}
}

pub type DemoHandle = Box<dyn DemoContext>;

struct StaticAssets;

impl AssetLoader for StaticAssets {
	fn read(&self, path: &str) -> Result<Vec<u8>, AssetError> {
		let bytes = match path {
			"font/font.json" => include_bytes!("../../../assets/font/font.json").as_slice(),
			"font/font.png" => include_bytes!("../../../assets/font/font.png").as_slice(),
			"mandelbrot/gradient.png" => include_bytes!("../../../assets/mandelbrot/gradient.png").as_slice(),
			"oldtree/texture.png" => include_bytes!("../../../assets/oldtree/texture.png").as_slice(),
			"screenmelt/e1m1.gif" => include_bytes!("../../../assets/screenmelt/e1m1.gif").as_slice(),
			"screenmelt/main-menu.png" => include_bytes!("../../../assets/screenmelt/main-menu.png").as_slice(),
			"textures/panel.png" => include_bytes!("../../../assets/textures/panel.png").as_slice(),
			"textures/panels.png" => include_bytes!("../../../assets/textures/panels.png").as_slice(),
			"textures/lapras.png" => include_bytes!("../../../assets/textures/lapras.png").as_slice(),
			"textures/2k_earth_daymap.jpg" => include_bytes!("../../../assets/textures/2k_earth_daymap.jpg").as_slice(),
			"textures/scene tiles.png" => include_bytes!("../../../assets/textures/scene tiles.png").as_slice(),
			"zeldawater/water.png" => include_bytes!("../../../assets/zeldawater/water.png").as_slice(),
			"zeldawater/distort.png" => include_bytes!("../../../assets/zeldawater/distort.png").as_slice(),
			_ => return Err(AssetError::missing(path)),
		};
		Ok(bytes.to_vec())
	}
}

pub struct SharedContext {
	webgl: shade::webgl::WebGLGraphics,
	demo: Box<dyn demos::DemoInterface>,
	screen_size: Vec2i,
	cursor: Vec2f,
	last_time: f64,
	pending_redraw: bool,
}

impl SharedContext {
	pub fn new(create: fn(&mut shade::Graphics, &dyn AssetLoader) -> Box<dyn demos::DemoInterface>) -> SharedContext {
		shade::webgl::setup_panic_hook();
		let mut webgl = shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		});
		let demo = create(webgl.as_graphics(), &StaticAssets);
		SharedContext {
			webgl,
			demo,
			screen_size: Vec2::ZERO,
			cursor: Vec2::ZERO,
			last_time: 0.0,
			pending_redraw: true,
		}
	}

	fn input(&mut self, input: Input) {
		let mut shell = WebShellServices {
			pending_redraw: &mut self.pending_redraw,
			time: self.last_time,
		};
		self.demo.input(input, self.webgl.as_graphics(), &mut shell);
	}

	fn file_opened_impl(&mut self, request_id: u32, path: Option<String>, bytes: Option<Vec<u8>>) {
		let mut shell = WebShellServices {
			pending_redraw: &mut self.pending_redraw,
			time: self.last_time,
		};
		self.demo.file_opened(request_id, path, bytes, self.webgl.as_graphics(), &mut shell);
	}
}

struct WebShellServices<'a> {
	pending_redraw: &'a mut bool,
	time: f64,
}

impl ShellServices for WebShellServices<'_> {
	fn get_time(&mut self) -> f64 {
		self.time
	}

	fn request_redraw(&mut self) {
		*self.pending_redraw = true;
	}

	fn set_cursor(&mut self, cursor: Cursor) {
		unsafe {
			set_cursor_import(web_cursor(cursor));
		}
	}

	fn open_file(&mut self, request: FileRequest) {
		let extensions = request.extensions.join(",");
		unsafe {
			open_file_import(request.id, request.title.as_ptr(), request.title.len(), extensions.as_ptr(), extensions.len());
		}
	}

	fn set_status(&mut self, text: &str) {
		unsafe {
			set_status_import(text.as_ptr(), text.len());
		}
	}
}

impl DemoContext for SharedContext {
	fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
		self.demo.resize(self.screen_size);
		self.pending_redraw = true;
	}

	fn draw(&mut self, time: f64) {
		let viewport = Bounds2::vec(self.screen_size);
		let dt = (time - self.last_time) as f32;
		let frame = Frame { viewport, time, dt };
		self.last_time = time;
		self.demo.draw(frame, self.webgl.as_graphics());
	}

	fn redraw_mode(&self) -> RedrawMode {
		self.demo.redraw_mode()
	}

	fn take_redraw_request(&mut self) -> bool {
		std::mem::take(&mut self.pending_redraw)
	}

	fn file_opened(&mut self, request_id: u32, path: Option<String>, bytes: Option<Vec<u8>>) {
		self.file_opened_impl(request_id, path, bytes);
	}

	fn mousemove(&mut self, dx: f32, dy: f32) {
		let delta = Vec2(dx, dy);
		self.cursor += delta;
		self.input(Input::MouseMove {
			position: self.cursor,
		});
	}

	fn mousedown(&mut self, button: u32) {
		self.input(Input::MouseButton {
			button: web_mouse_button(button),
			pressed: true,
			position: self.cursor,
		});
	}

	fn mouseup(&mut self, button: u32) {
		self.input(Input::MouseButton {
			button: web_mouse_button(button),
			pressed: false,
			position: self.cursor,
		});
	}

	fn wheel(&mut self, delta_y: f32) {
		self.input(Input::MouseWheel {
			delta: Vec2(0.0, delta_y),
			position: self.cursor,
		});
	}

	fn keydown(&mut self, key: u32) {
		self.input(Input::KeyDown(web_key(key)));
	}

	fn keyup(&mut self, key: u32) {
		self.input(Input::KeyUp(web_key(key)));
	}
}

fn web_key(key: u32) -> Key {
	match key {
		1 => Key::Digit1,
		2 => Key::Digit2,
		3 => Key::Digit3,
		10 => Key::ArrowLeft,
		11 => Key::ArrowRight,
		12 => Key::ArrowUp,
		13 => Key::ArrowDown,
		20 => Key::Shift,
		21 => Key::P,
		22 => Key::F2,
		23 => Key::Escape,
		_ => Key::Other,
	}
}

fn web_mouse_button(button: u32) -> gui::MouseButton {
	match button {
		0 => gui::MouseButton::LEFT,
		1 => gui::MouseButton::MIDDLE,
		2 => gui::MouseButton::RIGHT,
		other => gui::MouseButton(other as u8),
	}
}

fn web_cursor(cursor: Cursor) -> u32 {
	match cursor {
		Cursor::Default => 0,
		Cursor::Pointer => 1,
		Cursor::Grab => 2,
		Cursor::Grabbing => 3,
		Cursor::Crosshair => 4,
		Cursor::Move => 5,
		Cursor::ResizeHorizontal => 6,
		Cursor::ResizeVertical => 7,
		Cursor::ResizeNwse => 8,
		Cursor::ResizeNesw => 9,
	}
}
