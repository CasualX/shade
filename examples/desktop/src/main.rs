use std::{ffi, fs, mem, path, time};
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;
use shade::gui;

//----------------------------------------------------------------
// Winit and Glutin state.

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
		let width = NonZeroU32::new(size.width.max(1)).unwrap();
		let height = NonZeroU32::new(size.height.max(1)).unwrap();
		let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
			.with_srgb(Some(true))
			.build(raw_window_handle, width, height);
		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");
		let context = not_current
			.make_current(&surface)
			.expect("Failed NotCurrentContext.make_current");

		shade::gl::capi::load_with(|s| {
			let c = ffi::CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		GlWindow { size, window, surface, context }
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
		let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
		self.size = new_size;
		self.surface.resize(&self.context, width, height);
		self.window.request_redraw();
	}
}

//----------------------------------------------------------------
// Assets discovery and loader.

struct DiskAssets {
	root: path::PathBuf,
}

impl DiskAssets {
	fn new() -> DiskAssets {
		DiskAssets {
			root: path::PathBuf::from("assets"),
		}
	}
}

impl demos::AssetLoader for DiskAssets {
	fn read(&self, path: &str) -> Result<Vec<u8>, demos::AssetError> {
		let full_path = self.root.join(path);
		fs::read(&full_path).map_err(|source| demos::AssetError {
			path: full_path.display().to_string(),
			message: source.to_string(),
		})
	}
}

//----------------------------------------------------------------
// Shell state and services.

struct OpenedFile {
	request_id: u32,
	path: Option<String>,
	bytes: Option<Vec<u8>>,
}

struct ShellState {
	start: time::Instant,
	pending_file_open: Vec<OpenedFile>,
}

impl ShellState {
	fn new(start: time::Instant) -> ShellState {
		ShellState {
			start,
			pending_file_open: Vec::new(),
		}
	}
}

struct DesktopShell<'a> {
	window: &'a winit::window::Window,
	state: &'a mut ShellState,
}

impl demos::ShellServices for DesktopShell<'_> {
	fn get_time(&mut self) -> f64 {
		time::Instant::now().duration_since(self.state.start).as_secs_f64()
	}

	fn request_redraw(&mut self) {
		self.window.request_redraw();
	}

	fn open_file(&mut self, request: demos::FileRequest) {
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
			self.state.pending_file_open.push(OpenedFile {
				request_id: request.id,
				path: None,
				bytes: None,
			});
			return;
		};
		let bytes = fs::read(&path).ok();
		self.state.pending_file_open.push(OpenedFile {
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
			Cursor::ResizeHorizontal => winit::window::CursorIcon::EwResize,
			Cursor::ResizeVertical => winit::window::CursorIcon::NsResize,
			Cursor::ResizeNwse => winit::window::CursorIcon::NwseResize,
			Cursor::ResizeNesw => winit::window::CursorIcon::NeswResize,
		};
		self.window.set_cursor(cursor);
	}
}

//----------------------------------------------------------------
// Application state.

struct App {
	demo: Box<dyn demos::DemoInterface>,
	graphics: shade::gl::GlGraphics,
	window: GlWindow,
	shell: ShellState,
	last_frame: time::Instant,
	cursor: Vec2f,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>, demo_id: &str, assets: &DiskAssets) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut graphics = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let mut demo = {
			if let Some(demo) = demos::examples::find(demo_id) {
				(demo.create)(&mut graphics, assets)
			}
			else {
				eprintln!("Unknown demo '{demo_id}'. Available demos: {}", demos::examples::names_csv());
				std::process::exit(2);
			}
		};
		demo.resize(Vec2(size.width as i32, size.height as i32));
		let now = time::Instant::now();
		let shell = ShellState::new(now);
		Box::new(App { window, graphics, demo, shell, last_frame: now, cursor: Vec2::ZERO })
	}

	fn request_redraw(&self) {
		self.window.window.request_redraw();
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.window.resize(new_size);
		self.demo.resize(Vec2(new_size.width as i32, new_size.height as i32));
	}

	fn input(&mut self, input: demos::Input) {
		let mut shell = DesktopShell {
			window: &self.window.window,
			state: &mut self.shell,
		};
		self.demo.input(input, &mut self.graphics, &mut shell);

		loop {
			let pending_file_open = mem::take(&mut self.shell.pending_file_open);
			if pending_file_open.is_empty() {
				break;
			}
			for file in pending_file_open {
				let mut shell = DesktopShell {
					window: &self.window.window,
					state: &mut self.shell,
				};
				self.demo.file_opened(
					file.request_id,
					file.path,
					file.bytes,
					&mut self.graphics,
					&mut shell,
				);
			}
		}
	}

	fn mouse_move(&mut self, position: Vec2f) {
		self.cursor = position;
		self.input(demos::Input::MouseMove { position });
	}

	fn mouse_button(&mut self, button: gui::MouseButton, pressed: bool) {
		self.input(demos::Input::MouseButton {
			button,
			pressed,
			position: self.cursor,
		});
	}

	fn mouse_wheel(&mut self, delta: Vec2f) {
		self.input(demos::Input::MouseWheel {
			delta,
			position: self.cursor,
		});
	}

	fn draw(&mut self) {
		let now = time::Instant::now();
		let viewport = Bounds2!(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		let frame = demos::Frame {
			viewport,
			time: now.duration_since(self.shell.start).as_secs_f64(),
			dt: now.duration_since(self.last_frame).as_secs_f32(),
		};
		self.last_frame = now;
		self.demo.draw(frame, &mut self.graphics);
	}

	fn swap_buffers(&self) {
		self.window.surface.swap_buffers(&self.window.context).unwrap();
	}

	fn redraw_mode(&self) -> demos::RedrawMode {
		self.demo.redraw_mode()
	}
}

//----------------------------------------------------------------
// Main loop.

fn mouse_button(button: winit::event::MouseButton) -> gui::MouseButton {
	match button {
		winit::event::MouseButton::Left => gui::MouseButton::LEFT,
		winit::event::MouseButton::Right => gui::MouseButton::RIGHT,
		winit::event::MouseButton::Middle => gui::MouseButton::MIDDLE,
		winit::event::MouseButton::Back => gui::MouseButton(4),
		winit::event::MouseButton::Forward => gui::MouseButton(5),
		winit::event::MouseButton::Other(button) => gui::MouseButton(button as u8),
	}
}

fn key(physical_key: winit::keyboard::PhysicalKey) -> demos::Key {
	use winit::keyboard::{KeyCode, PhysicalKey};
	match physical_key {
		PhysicalKey::Code(KeyCode::Digit1) | PhysicalKey::Code(KeyCode::Numpad1) => demos::Key::Digit1,
		PhysicalKey::Code(KeyCode::Digit2) | PhysicalKey::Code(KeyCode::Numpad2) => demos::Key::Digit2,
		PhysicalKey::Code(KeyCode::Digit3) | PhysicalKey::Code(KeyCode::Numpad3) => demos::Key::Digit3,
		PhysicalKey::Code(KeyCode::ArrowLeft) => demos::Key::ArrowLeft,
		PhysicalKey::Code(KeyCode::ArrowRight) => demos::Key::ArrowRight,
		PhysicalKey::Code(KeyCode::ArrowUp) => demos::Key::ArrowUp,
		PhysicalKey::Code(KeyCode::ArrowDown) => demos::Key::ArrowDown,
		PhysicalKey::Code(KeyCode::F2) => demos::Key::F2,
		PhysicalKey::Code(KeyCode::KeyP) => demos::Key::P,
		PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => demos::Key::Shift,
		PhysicalKey::Code(KeyCode::Escape) => demos::Key::Escape,
		_ => demos::Key::Other,
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
					let new_app = App::new(event_loop, size, &demo_id, &assets);
					new_app.request_redraw();
					app = Some(new_app);
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
						app.mouse_move(position);
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.mouse_button(mouse_button(button), state == ElementState::Pressed);
					}
				}
				WindowEvent::MouseWheel { delta, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let y = match delta {
							MouseScrollDelta::LineDelta(_, y) => y * 16.0,
							MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
						};
						app.mouse_wheel(Vec2(0.0, y));
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let key = key(event.physical_key);
						let input = match event.state {
							ElementState::Pressed if !event.repeat => Some(demos::Input::KeyDown(key)),
							ElementState::Released => Some(demos::Input::KeyUp(key)),
							_ => None,
						};
						if key == demos::Key::Escape && event.state == ElementState::Pressed {
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
						app.swap_buffers();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					if app.redraw_mode() == demos::RedrawMode::Continuous {
						app.request_redraw();
					}
				}
			}
			_ => {}
		}
	});
}
