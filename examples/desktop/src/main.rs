use std::ffi::CString;
use std::fs;
use std::mem;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use glutin::prelude::*;
use shade::cvmath::*;
use demos::*;

struct GlWindow {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
}

impl GlWindow {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> GlWindow {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);
		let window_attributes = winit::window::WindowAttributes::default()
			.with_title("shade examples")
			.with_inner_size(size);
		let config_picker = |configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>| {
			configs
				.filter(|c| c.srgb_capable())
				.max_by_key(|c| c.num_samples())
				.expect("No GL configs found")
		};
		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template_builder, config_picker)
			.expect("Failed DisplayBuilder.build");
		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window
			.window_handle()
			.expect("Failed Window.window_handle")
			.as_raw();
		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));
		let gl_display = gl_config.display();
		let not_current = unsafe { gl_display.create_context(&gl_config, &context_attributes) }
			.expect("Failed Display.create_context");
		let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
			.with_srgb(Some(true))
			.build(
				raw_window_handle,
				NonZeroU32::new(size.width.max(1)).unwrap(),
				NonZeroU32::new(size.height.max(1)).unwrap(),
			);
		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");
		let context = not_current
			.make_current(&surface)
			.expect("Failed NotCurrentContext.make_current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		GlWindow { size, window, surface, context }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
	}
}

struct DiskAssets {
	root: PathBuf,
}

impl DiskAssets {
	fn new() -> DiskAssets {
		DiskAssets {
			root: PathBuf::from("assets"),
		}
	}
}

impl AssetLoader for DiskAssets {
	fn read(&self, path: &str) -> Result<Vec<u8>, AssetError> {
		let full_path = self.root.join(path);
		fs::read(&full_path).map_err(|source| AssetError {
			path: full_path.display().to_string(),
			message: source.to_string(),
		})
	}
}

struct DesktopServices<'a> {
	window: &'a winit::window::Window,
	start: Instant,
	pending_file_open: Vec<OpenedFile>,
	redraw: bool,
}

struct OpenedFile {
	request_id: u32,
	path: Option<String>,
	bytes: Option<Vec<u8>>,
}

impl<'a> DesktopServices<'a> {
	fn new(window: &'a winit::window::Window, start: Instant) -> DesktopServices<'a> {
		DesktopServices {
			window,
			start,
			pending_file_open: Vec::new(),
			redraw: false,
		}
	}
}

impl ShellServices for DesktopServices<'_> {
	fn get_time(&mut self) -> f64 {
		Instant::now().duration_since(self.start).as_secs_f64()
	}

	fn request_redraw(&mut self) {
		self.redraw = true;
	}

	fn open_file(&mut self, request: FileRequest) {
		let patterns: Vec<String> = request.extensions.iter().map(|ext| format!("*.{ext}")).collect();
		let patterns: Vec<&str> = patterns.iter().map(String::as_str).collect();
		let filter = rustydialogs::FileFilter {
			desc: "Files",
			patterns: &patterns,
		};
		let Some(path) = (rustydialogs::FileDialog {
			title: request.title,
			path: None,
			filter: Some(&[filter]),
			owner: Some(self.window),
		}).pick_file() else {
			self.pending_file_open.push(OpenedFile {
				request_id: request.id,
				path: None,
				bytes: None,
			});
			return;
		};
		let bytes = fs::read(&path).ok();
		self.pending_file_open.push(OpenedFile {
			request_id: request.id,
			path: Some(path.display().to_string()),
			bytes,
		});
	}

	fn set_status(&mut self, text: &str) {
		eprintln!("{text}");
	}

	fn set_cursor(&mut self, cursor: demos::Cursor) {
		use demos::Cursor;
		let cursor = match cursor {
			Cursor::Default => winit::window::CursorIcon::Default,
			Cursor::Pointer => winit::window::CursorIcon::Pointer,
			Cursor::Grab => winit::window::CursorIcon::Grab,
			Cursor::Grabbing => winit::window::CursorIcon::Grabbing,
			Cursor::Crosshair => winit::window::CursorIcon::Crosshair,
			Cursor::Move => winit::window::CursorIcon::Move,
			Cursor::ResizeEastWest => winit::window::CursorIcon::EwResize,
			Cursor::ResizeNorthSouth => winit::window::CursorIcon::NsResize,
			Cursor::ResizeNwse => winit::window::CursorIcon::NwseResize,
			Cursor::ResizeNesw => winit::window::CursorIcon::NeswResize,
		};
		self.window.set_cursor(cursor);
	}
}

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: Box<dyn DemoInterface>,
	start: Instant,
	last_frame: Instant,
	cursor: Vec2f,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>, demo_id: &str, assets: &DiskAssets) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let mut demo = create_demo(demo_id, opengl.as_graphics(), assets);
		demo.resize(Vec2(size.width as i32, size.height as i32));
		let now = Instant::now();
		Box::new(App { window, opengl, demo, start: now, last_frame: now, cursor: Vec2::ZERO })
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.window.resize(new_size);
		self.demo.resize(Vec2(new_size.width as i32, new_size.height as i32));
		self.window.window.request_redraw();
	}

	fn input(&mut self, input: Input) {
		let mut services = DesktopServices::new(&self.window.window, self.start);
		self.demo.input(input, self.opengl.as_graphics(), &mut services);
		for file in mem::take(&mut services.pending_file_open) {
			self.demo.file_opened(file.request_id, file.path, file.bytes, self.opengl.as_graphics(), &mut services);
		}
		if services.redraw {
			self.window.window.request_redraw();
		}
	}

	fn draw(&mut self) {
		let now = Instant::now();
		let viewport = Bounds2!(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		let frame = Frame {
			viewport,
			time: now.duration_since(self.start).as_secs_f64(),
			dt: now.duration_since(self.last_frame).as_secs_f32(),
		};
		self.last_frame = now;
		self.demo.draw(frame, self.opengl.as_graphics());
	}
}

fn create_demo(demo_id: &str, g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	if let Some(demo) = demos::examples::find(demo_id) {
		return (demo.create)(g, assets);
	}

	eprintln!("Unknown demo '{demo_id}'. Available demos: {}", demos::examples::names_csv());
	std::process::exit(2);
}

fn mouse_button(button: winit::event::MouseButton) -> MouseButton {
	match button {
		winit::event::MouseButton::Left => MouseButton::Left,
		winit::event::MouseButton::Right => MouseButton::Right,
		winit::event::MouseButton::Middle => MouseButton::Middle,
		winit::event::MouseButton::Back => MouseButton::Other(4),
		winit::event::MouseButton::Forward => MouseButton::Other(5),
		winit::event::MouseButton::Other(button) => MouseButton::Other(button),
	}
}

fn key(physical_key: winit::keyboard::PhysicalKey) -> Key {
	use winit::keyboard::{KeyCode, PhysicalKey};
	match physical_key {
		PhysicalKey::Code(KeyCode::Digit1) | PhysicalKey::Code(KeyCode::Numpad1) => Key::Digit1,
		PhysicalKey::Code(KeyCode::Digit2) | PhysicalKey::Code(KeyCode::Numpad2) => Key::Digit2,
		PhysicalKey::Code(KeyCode::Digit3) | PhysicalKey::Code(KeyCode::Numpad3) => Key::Digit3,
		PhysicalKey::Code(KeyCode::ArrowLeft) => Key::ArrowLeft,
		PhysicalKey::Code(KeyCode::ArrowRight) => Key::ArrowRight,
		PhysicalKey::Code(KeyCode::ArrowUp) => Key::ArrowUp,
		PhysicalKey::Code(KeyCode::ArrowDown) => Key::ArrowDown,
		PhysicalKey::Code(KeyCode::F2) => Key::F2,
		PhysicalKey::Code(KeyCode::KeyP) => Key::P,
		PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => Key::Shift,
		PhysicalKey::Code(KeyCode::Escape) => Key::Escape,
		_ => Key::Other,
	}
}

fn main() {
	let Some(demo_id) = std::env::args().nth(1) else {
		println!("Available demos:");
		for demo_name in demos::examples::names() {
			println!("{demo_name}");
		}
		return;
	};
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
	let size = winit::dpi::PhysicalSize::new(960, 720);
	let assets = DiskAssets::new();
	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, MouseScrollDelta, WindowEvent};

		match event {
			Event::Resumed => {
				if app.is_none() {
					let app_new = App::new(event_loop, size, &demo_id, &assets);
					app_new.window.window.request_redraw();
					app = Some(app_new);
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.resize(new_size);
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let position = Vec2(position.x as f32, position.y as f32);
						app.cursor = position;
						app.input(Input::MouseMove { position });
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.input(Input::MouseButton {
							button: mouse_button(button),
							pressed: state == ElementState::Pressed,
							position: app.cursor,
						});
					}
				}
				WindowEvent::MouseWheel { delta, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let y = match delta {
							MouseScrollDelta::LineDelta(_, y) => y * 16.0,
							MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
						};
						app.input(Input::MouseWheel {
							delta: Vec2(0.0, y),
							position: app.cursor,
						});
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let key = key(event.physical_key);
						let input = match event.state {
							ElementState::Pressed if !event.repeat => Some(Input::KeyDown(key)),
							ElementState::Released => Some(Input::KeyUp(key)),
							_ => None,
						};
						if key == Key::Escape && event.state == ElementState::Pressed {
							event_loop.exit();
						}
						if let Some(input) = input {
							app.input(input);
						}
					}
				}
				WindowEvent::CloseRequested => event_loop.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						app.draw();
						app.window.surface.swap_buffers(&app.window.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					if app.demo.redraw_mode() == RedrawMode::Continuous {
						app.window.window.request_redraw();
					}
				}
			}
			_ => {}
		}
	});
}
