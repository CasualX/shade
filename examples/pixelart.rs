use std::ffi::CString;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time;

use glutin::prelude::*;
use shade::cvmath::*;
use shade::d2;

//----------------------------------------------------------------
// OpenGL window wrapper

struct GlWindow {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
}

impl GlWindow {
	fn new(
		event_loop: &winit::event_loop::ActiveEventLoop,
		size: winit::dpi::PhysicalSize<u32>,
	) -> GlWindow {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let template_builder = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_title("shade pixel art")
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

		let surface_attributes_builder = SurfaceAttributesBuilder::<WindowSurface>::new()
			.with_srgb(Some(true));
		let surface_attributes = surface_attributes_builder.build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
			.expect("Failed Display.create_window_surface");

		let context = not_current.make_current(&surface)
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

//----------------------------------------------------------------
// Demo state

const DEFAULT_IMAGE: &str = "examples/textures/lapras.png";

#[derive(Copy, Clone, Eq, PartialEq)]
enum FilterMode {
	Nearest,
	Linear,
	PixelArt,
}

impl FilterMode {
	fn label(self) -> &'static str {
		match self {
			FilterMode::Nearest => "nearest",
			FilterMode::Linear => "linear",
			FilterMode::PixelArt => "pixel art",
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum PostProcessMode {
	None,
	Crt,
}

impl PostProcessMode {
	fn next(self) -> PostProcessMode {
		match self {
			PostProcessMode::None => PostProcessMode::Crt,
			PostProcessMode::Crt => PostProcessMode::None,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DragMode {
	None,
	Pan,
	TurnZoom,
}

struct PixelArtDemo {
	epoch: time::Instant,
	textured_shader: shade::ShaderProgram,
	pixelart_shader: shade::ShaderProgram,
	pp: shade::d2::PostProcessQuad,
	pp_copy_shader: shade::ShaderProgram,
	pp_crt_shader: shade::ShaderProgram,
	hud_font: d2::FontResource<shade::msdfgen::Font>,
	nearest_texture: shade::Texture2D,
	linear_texture: shade::Texture2D,
	render_texture: shade::Texture2D,
	image_path: PathBuf,
	image_size: Vec2f,
	filter_mode: FilterMode,
	post_process_mode: PostProcessMode,
	pan: Vec2f,
	zoom: f32,
	rotation: f32,
	cursor: Vec2f,
	drag_mode: DragMode,
	last_drag_cursor: Vec2f,
}

impl PixelArtDemo {
	fn new(g: &mut shade::Graphics) -> PixelArtDemo {
		let epoch = time::Instant::now();
		let textured_shader = g.shader_compile(
			shade::shaders::glsl330core::TEXTURED_VS,
			shade::shaders::glsl330core::TEXTURED_FS,
		);
		let pixelart_shader = g.shader_compile(
			shade::shaders::glsl330core::PIXELART_VS,
			shade::shaders::glsl330core::PIXELART_FS,
		);
		let hud_font = {
			let font: shade::msdfgen::FontDto = serde_json::from_str(include_str!("font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();
			let texture = {
				let image = shade::image::DecodedImage::load_file_png("examples/font/font.png").unwrap().to_rgba()
					.map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
				g.image(&image)
			};
			let shader = g.shader_compile(shade::shaders::glsl330core::MTSDF_VS, shade::shaders::glsl330core::MTSDF_FS);
			d2::FontResource { font, texture, shader }
		};
		let pp = shade::d2::PostProcessQuad::create(g);
		let pp_copy_shader = g.shader_compile(
			shade::shaders::glsl330core::POST_PROCESS_VS,
			shade::shaders::glsl330core::POST_PROCESS_COPY_FS,
		);
		let pp_crt_shader = g.shader_compile(
			shade::shaders::glsl330core::POST_PROCESS_VS,
			shade::shaders::glsl330core::POST_PROCESS_CRT_FS,
		);

		let mut demo = PixelArtDemo {
			epoch,
			textured_shader,
			pixelart_shader,
			pp,
			pp_copy_shader,
			pp_crt_shader,
			hud_font,
			nearest_texture: shade::Texture2D::INVALID,
			linear_texture: shade::Texture2D::INVALID,
			render_texture: shade::Texture2D::INVALID,
			image_path: PathBuf::from(DEFAULT_IMAGE),
			image_size: Vec2(1.0, 1.0),
			filter_mode: FilterMode::PixelArt,
			post_process_mode: PostProcessMode::Crt,
			pan: Vec2::ZERO,
			zoom: 1.0,
			rotation: 0.0,
			cursor: Vec2::ZERO,
			drag_mode: DragMode::None,
			last_drag_cursor: Vec2::ZERO,
		};
		demo.load_image(g, Path::new(DEFAULT_IMAGE)).expect("Failed to load default image");
		demo
	}

	fn image_name(&self) -> &str {
		self.image_path
			.file_name()
			.and_then(|name| name.to_str())
			.unwrap_or("image")
	}

	fn shader(&self) -> shade::ShaderProgram {
		match self.filter_mode {
			FilterMode::Nearest | FilterMode::Linear => self.textured_shader,
			FilterMode::PixelArt => self.pixelart_shader,
		}
	}

	fn ensure_render_texture(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let info = shade::Texture2DInfo {
			format: shade::TextureFormat::SRGBA8,
			width: viewport.width().max(1),
			height: viewport.height().max(1),
			props: shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage!(WRITE | SAMPLED | COLOR_TARGET),
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Edge,
				wrap_v: shade::TextureWrap::Edge,
				..Default::default()
			},
		};
		self.render_texture = g.texture2d_update(self.render_texture, &info);
	}

	fn texture(&self) -> shade::Texture2D {
		match self.filter_mode {
			FilterMode::Nearest => self.nearest_texture,
			FilterMode::Linear => self.linear_texture,
			FilterMode::PixelArt => self.linear_texture,
		}
	}

	fn load_image(&mut self, g: &mut shade::Graphics, path: &Path) -> Result<(), String> {
		let image = shade::image::DecodedImage::load_file(path)
			.map_err(|err| format!("{}: {err:?}", path.display()))?;

		let nearest_props = shade::TextureProps {
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			..Default::default()
		};
		let linear_props = shade::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			..Default::default()
		};
		let nearest_texture = g.image(&(&image, &nearest_props));
		let linear_texture = g.image(&(&image, &linear_props));
		if self.nearest_texture != shade::Texture2D::INVALID {
			g.release(self.nearest_texture);
		}
		if self.linear_texture != shade::Texture2D::INVALID {
			g.release(self.linear_texture);
		}

		self.nearest_texture = nearest_texture;
		self.linear_texture = linear_texture;
		self.image_path = path.to_path_buf();
		self.image_size = Vec2(image.width() as f32, image.height() as f32);
		self.pan = Vec2::ZERO;
		self.zoom = 1.0;
		self.rotation = 0.0;
		Ok(())
	}

	fn begin_drag(&mut self, drag_mode: DragMode) {
		self.drag_mode = drag_mode;
		self.last_drag_cursor = self.cursor;
	}

	fn end_drag(&mut self) {
		self.drag_mode = DragMode::None;
	}

	fn drag_to_cursor(&mut self) {
		let delta = self.cursor - self.last_drag_cursor;
		self.last_drag_cursor = self.cursor;

		match self.drag_mode {
			DragMode::None => {}
			DragMode::Pan => {
				let zoom = self.zoom.max(1.0 / 64.0);
				let delta = delta / zoom;
				let sin_theta = self.rotation.sin();
				let cos_theta = self.rotation.cos();
				self.pan += Vec2(
					delta.x * cos_theta + delta.y * sin_theta,
					-delta.x * sin_theta + delta.y * cos_theta,
				);
			}
			DragMode::TurnZoom => {
				self.rotation += delta.x * 0.01;
				self.apply_zoom_delta(-delta.y * 0.01);
			}
		}
	}

	fn apply_zoom_delta(&mut self, delta: f32) {
		self.zoom = (self.zoom * delta.exp()).clamp(1.0 / 64.0, 64.0);
	}

	fn open_file_dialog(&self, window: &winit::window::Window) -> Option<PathBuf> {
		let filters = [
			rustydialogs::FileFilter {
				desc: "Images",
				patterns: &["*.png", "*.gif", "*.jpg", "*.jpeg"],
			},
		];

		rustydialogs::FileDialog {
			title: "Open pixel art image",
			path: Some(self.image_path.as_path()),
			filter: Some(&filters),
			owner: Some(window),
		}.pick_file()
	}

	fn draw_scene(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		self.ensure_render_texture(g, viewport);
		let target_viewport = Bounds2!(0, 0, viewport.width().max(1), viewport.height().max(1));
		g.begin(&shade::BeginArgs::Immediate {
			viewport: target_viewport,
			color: &[self.render_texture],
			levels: None,
			depth: shade::Texture2D::INVALID,
		});
		shade::clear!(g, color: Vec4(0.08, 0.09, 0.10, 1.0));

		let mut cv = shade::d2::TexturedBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.shader = self.shader();
		cv.uniform.transform = Transform2::ortho(Bounds2!(0.0, 0.0, viewport.width() as f32, viewport.height() as f32));
		cv.uniform.texture = self.texture();

		let screen_center = Vec2(viewport.width() as f32 * 0.5, viewport.height() as f32 * 0.5);
		let half = Vec2(self.image_size.x * 0.5, self.image_size.y * 0.5);
		let rotate = Transform2::rotation(Angle(self.rotation)).around(screen_center);

		let corners = [
			(Vec2(0.0, 0.0), Vec2(-half.x, -half.y)),
			(Vec2(0.0, 1.0), Vec2(-half.x, half.y)),
			(Vec2(1.0, 1.0), Vec2(half.x, half.y)),
			(Vec2(1.0, 0.0), Vec2(half.x, -half.y)),
		];

		{
			let mut prim = cv.begin(shade::PrimType::Triangles, 4, 2);
			for &(uv, local) in &corners {
				let scaled = (self.pan + local) * self.zoom;
				prim.add_vertex(shade::d2::TexturedVertex {
					pos: rotate * (screen_center + scaled),
					uv,
					color: Vec4::dup(255),
				});
			}
			prim.add_indices_quad();
		}

		cv.draw(g);
		g.end();
	}

	fn draw_hud(&self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let mut hud = d2::TextBuffer::new();
		hud.blend_mode = shade::BlendMode::Alpha;
		hud.uniform.transform = Transform2::ortho(Bounds2!(0.0, 0.0, viewport.width() as f32, viewport.height() as f32));
		hud.uniform.outline_width_relative = 0.2;
		let mut pos = Vec2(16.0, 18.0);
		let mut scribe = d2::Scribe {
			font_size: 22.0,
			line_height: 28.0,
			x_pos: pos.x,
			color: Vec4(245, 245, 245, 255),
			..Default::default()
		};
		scribe.set_baseline_relative(0.0);
		let hud_text = format!(
			"image: {}\nmode: {}\npost fx: {}\nzoom: {:.2}x\nrotation: {:.1} deg\n1 = nearest  2 = linear  3 = pixel art\nP = cycle post fx\nleft drag = pan\nright drag = rotate + zoom\nwheel = zoom\nF2 = open image",
			self.image_name(),
			self.filter_mode.label(),
			match self.post_process_mode {
				PostProcessMode::None => "none",
				PostProcessMode::Crt => "crt",
			},
			self.zoom,
			self.rotation.to_degrees(),
		);
		hud.text_write(&self.hud_font, &mut scribe, &mut pos, &hud_text);
		hud.draw(g);
	}

	fn draw(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		self.draw_scene(g, viewport);

		let elapsed = self.epoch.elapsed().as_secs_f32();
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.0, 0.0, 0.0, 1.0));

		match self.post_process_mode {
			PostProcessMode::None => self.pp.draw(g,
				self.pp_copy_shader,
				shade::BlendMode::Solid,
				&[&shade::shaders::PostProcessCopyUniforms {
					texture: self.render_texture,
				}],
			),
			PostProcessMode::Crt => self.pp.draw(g,
				self.pp_crt_shader,
				shade::BlendMode::Solid,
				&[&shade::shaders::PostProcessCrtUniforms {
					texture: self.render_texture,
					scanline_count: viewport.height() as f32 * 0.25,
					time: elapsed,
					..Default::default()
				}],
			),
		}

		self.draw_hud(g, viewport);
		g.end();
	}
}

struct App {
	window: GlWindow,
	opengl: shade::gl::GlGraphics,
	demo: PixelArtDemo,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop, size: winit::dpi::PhysicalSize<u32>) -> Box<App> {
		let window = GlWindow::new(event_loop, size);
		let mut opengl = shade::gl::GlGraphics::new(shade::gl::GlConfig { srgb: true });
		let demo = PixelArtDemo::new(opengl.as_graphics());
		Box::new(App { window, opengl, demo })
	}

	fn draw(&mut self) {
		let viewport = Bounds2!(0, 0, self.window.size.width as i32, self.window.size.height as i32);
		self.demo.draw(self.opengl.as_graphics(), viewport);
	}

}

//----------------------------------------------------------------

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
	let size = winit::dpi::PhysicalSize::new(960, 720);

	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

		match event {
			Event::Resumed => {
				if app.is_none() {
					app = Some(App::new(event_loop, size));
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						app.window.resize(new_size);
						app.window.window.request_redraw();
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.demo.cursor = Vec2(position.x as f32, position.y as f32);
						if app.demo.drag_mode != DragMode::None {
							app.demo.drag_to_cursor();
							app.window.window.request_redraw();
						}
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					if let Some(app) = app.as_deref_mut() {
						match (button, state) {
							(MouseButton::Left, ElementState::Pressed) => app.demo.begin_drag(DragMode::Pan),
							(MouseButton::Right, ElementState::Pressed) => app.demo.begin_drag(DragMode::TurnZoom),
							(MouseButton::Left | MouseButton::Right, ElementState::Released) => app.demo.end_drag(),
							_ => {}
						}
						app.window.window.request_redraw();
					}
				}
				WindowEvent::MouseWheel { delta, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let zoom_delta = match delta {
							MouseScrollDelta::LineDelta(_, y) => y * 0.12,
							MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.0025,
						};
						app.demo.apply_zoom_delta(zoom_delta);
						app.window.window.request_redraw();
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					if event.state == ElementState::Pressed && !event.repeat {
						if let Some(app) = app.as_deref_mut() {
							match event.physical_key {
								PhysicalKey::Code(KeyCode::Digit1) => app.demo.filter_mode = FilterMode::Nearest,
								PhysicalKey::Code(KeyCode::Digit2) => app.demo.filter_mode = FilterMode::Linear,
								PhysicalKey::Code(KeyCode::Digit3) => app.demo.filter_mode = FilterMode::PixelArt,
								PhysicalKey::Code(KeyCode::KeyP) => app.demo.post_process_mode = app.demo.post_process_mode.next(),
								PhysicalKey::Code(KeyCode::F2) => {
									if let Some(path) = app.demo.open_file_dialog(&app.window.window) {
										if let Err(err) = app.demo.load_image(app.opengl.as_graphics(), &path) {
											eprintln!("{err}");
										}
									}
								}
								PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
								_ => {}
							}
							app.window.window.request_redraw();
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
					app.window.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}