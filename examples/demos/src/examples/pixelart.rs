use crate::*;

const DEFAULT_IMAGE: &str = "textures/lapras.png";
const OPEN_IMAGE_REQUEST: u32 = 1;
const TURN_ZOOM_MIN_RADIUS: f32 = 24.0;

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

#[derive(Copy, Clone, PartialEq)]
enum DragMode {
	None,
	Pan,
	TurnZoom(TurnZoomGesture),
}

#[derive(Copy, Clone, PartialEq)]
struct TurnZoomGesture {
	screen_center: Vec2f,
	basis: Vec2f,
	start_zoom: f32,
	start_rotation: Anglef,
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(PixelArt::new(g, assets))
}

struct PixelArt {
	textured_shader: Box<dyn shade::ShaderProgram>,
	pixelart_shader: Box<dyn shade::ShaderProgram>,
	pp: shade::d2::PostProcessQuad,
	pp_copy_shader: Box<dyn shade::ShaderProgram>,
	pp_crt_shader: Box<dyn shade::ShaderProgram>,
	line_shader: Box<dyn shade::ShaderProgram>,
	hud_font: d2::FontResource<shade::msdfgen::Font>,
	nearest_texture: Option<Box<dyn shade::Texture2D>>,
	linear_texture: Option<Box<dyn shade::Texture2D>>,
	render_texture: Option<Box<dyn shade::Texture2D>>,
	image_name: String,
	image_size: Vec2f,
	viewport: Bounds2i,
	filter_mode: FilterMode,
	post_process_mode: PostProcessMode,
	pan: Vec2f,
	zoom: f32,
	rotation: Anglef,
	cursor: Vec2f,
	drag_mode: DragMode,
}

impl PixelArt {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> PixelArt {
		let mut shader_source = shade::shader_interface! {
			files {
				"textured.glsl" => shade::shaders::TEXTURED,
				"pixelart.glsl" => shade::shaders::PIXELART,
				"color.glsl" => shade::shaders::COLOR,
				"mtsdf.glsl" => shade::shaders::MTSDF,
				"post_process.copy.glsl" => shade::shaders::POST_PROCESS_COPY,
				"post_process.crt.glsl" => shade::shaders::POST_PROCESS_CRT,
			}
		};
		let textured_shader = g.shader_compile(&mut shader_source, "textured.glsl", &[]);
		let pixelart_shader = g.shader_compile(&mut shader_source, "pixelart.glsl", &[]);
		let line_shader = g.shader_compile(&mut shader_source, "color.glsl", &[]);
		let hud_font = load_font(g, assets, false);
		let pp = shade::d2::PostProcessQuad::create(g);
		let pp_copy_shader = g.shader_compile(&mut shader_source, "post_process.copy.glsl", &[]);
		let pp_crt_shader = g.shader_compile(&mut shader_source, "post_process.crt.glsl", &[]);

		let mut demo = PixelArt {
			textured_shader,
			pixelart_shader,
			pp,
			pp_copy_shader,
			pp_crt_shader,
			line_shader,
			hud_font,
			nearest_texture: None,
			linear_texture: None,
			render_texture: None,
			image_name: "lapras.png".to_owned(),
			image_size: Vec2(1.0, 1.0),
			viewport: Bounds2!(0, 0, 1, 1),
			filter_mode: FilterMode::PixelArt,
			post_process_mode: PostProcessMode::Crt,
			pan: Vec2::ZERO,
			zoom: 1.0,
			rotation: Anglef::ZERO,
			cursor: Vec2::ZERO,
			drag_mode: DragMode::None,
		};
		let image = assets.read(DEFAULT_IMAGE).unwrap();
		demo.load_image_bytes(g, Some("lapras.png".to_owned()), &image).unwrap();
		demo
	}

	fn shader(&self) -> &dyn shade::ShaderProgram {
		match self.filter_mode {
			FilterMode::Nearest | FilterMode::Linear => &*self.textured_shader,
			FilterMode::PixelArt => &*self.pixelart_shader,
		}
	}

	fn texture(&self) -> &dyn shade::Texture2D {
		match self.filter_mode {
			FilterMode::Nearest => &**self.nearest_texture.as_ref().expect("nearest texture is not loaded"),
			FilterMode::Linear | FilterMode::PixelArt => &**self.linear_texture.as_ref().expect("linear texture is not loaded"),
		}
	}

	fn ensure_render_texture(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let info = shade::Texture2DInfo {
			format: shade::TextureFormat::SRGBA8,
			width: viewport.width(),
			height: viewport.height(),
			props: shade::TextureProps! {
				usage: shade::TextureUsage!(WRITE | SAMPLED | COLOR_TARGET),
				filter: shade::TextureFilter::Linear,
				wrap: shade::TextureWrap::Edge,
			},
		};
		if let Some(texture) = &mut self.render_texture {
			g.texture2d_update(&mut **texture, &info);
		}
		else {
			self.render_texture = Some(g.texture2d_create(&info));
		}
	}

	fn load_image_bytes(&mut self, g: &mut shade::Graphics, path: Option<String>, bytes: &[u8]) -> Result<(), String> {
		let image = shade::image::DecodedImage::load_memory(bytes).map_err(|err| format!("{err:?}"))?;
		let nearest_props = shade::TextureProps! {
			filter: shade::TextureFilter::Nearest,
		};
		let linear_props = shade::TextureProps! {
			filter: shade::TextureFilter::Linear,
		};
		let nearest_texture = g.image(&nearest_props.bind(&image));
		let linear_texture = g.image(&linear_props.bind(&image));
		self.nearest_texture = Some(nearest_texture);
		self.linear_texture = Some(linear_texture);
		self.image_name = path.as_deref().and_then(|path| path.rsplit(['/', '\\']).next()).unwrap_or("image").to_owned();
		self.image_size = Vec2(image.width() as f32, image.height() as f32);
		self.pan = Vec2::ZERO;
		self.zoom = 1.0;
		self.rotation = Anglef::ZERO;
		Ok(())
	}

	fn apply_drag_delta(&mut self, delta: Vec2f) {
		match self.drag_mode {
			DragMode::None => {}
			DragMode::Pan => {
				let zoom = self.zoom.max(1.0 / 64.0);
				let delta = Transform2::rotation(-self.rotation) * (delta / zoom);
				self.pan += delta;
			}
			DragMode::TurnZoom(_) => {}
		}
	}

	fn apply_zoom_delta(&mut self, delta: f32) {
		self.zoom = (self.zoom * delta.exp()).clamp(1.0 / 64.0, 64.0);
	}

	fn begin_turn_zoom(&mut self, viewport: Bounds2i) {
		let screen_center = viewport.cast::<f32>().center();
		let basis = self.cursor - screen_center;
		if basis.len() < TURN_ZOOM_MIN_RADIUS {
			self.drag_mode = DragMode::None;
			return;
		}
		self.drag_mode = DragMode::TurnZoom(TurnZoomGesture {
			screen_center,
			basis,
			start_zoom: self.zoom,
			start_rotation: self.rotation,
		});
	}

	fn update_turn_zoom(&mut self) {
		let DragMode::TurnZoom(gesture) = self.drag_mode else {
			return;
		};
		let current = self.cursor - gesture.screen_center;
		let (current_dir, current_len) = current.norm_len();
		if current_len < TURN_ZOOM_MIN_RADIUS {
			return;
		}
		let (basis_dir, basis_len) = gesture.basis.norm_len();
		self.zoom = (gesture.start_zoom * (current_len / basis_len)).clamp(1.0 / 64.0, 64.0);
		self.rotation = gesture.start_rotation + basis_dir.signed_angle(current_dir);
	}

	fn draw_turn_zoom_overlay(&self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let DragMode::TurnZoom(gesture) = self.drag_mode else {
			return;
		};
		let mut buf = d2::DrawBuilder::<d2::ColorVertex, d2::ColorUniform>::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		buf.shader = Some(&*self.line_shader);

		let basis_pen = d2::Pen {
			template: d2::ColorTemplate {
				color1: Vec4(255, 210, 0, 220),
				color2: Vec4(255, 210, 0, 220),
			},
		};
		let current_pen = d2::Pen {
			template: d2::ColorTemplate {
				color1: Vec4(80, 220, 255, 220),
				color2: Vec4(80, 220, 255, 220),
			},
		};

		buf.draw_line(&basis_pen, gesture.screen_center, gesture.screen_center + gesture.basis);
		buf.draw_line(&current_pen, gesture.screen_center, self.cursor);
		buf.draw(g);
	}

	fn draw_scene(&mut self, g: &mut shade::Graphics, viewport: Bounds2i) {
		self.ensure_render_texture(g, viewport);
		let render_texture = &**self.render_texture.as_ref().expect("render texture was not created");
		g.begin(&shade::BeginArgs::Immediate {
			viewport,
			color: &[render_texture],
			levels: None,
			depth: None,
		});
		shade::clear!(g, color: Vec4(0.08, 0.09, 0.10, 1.0));

		let mut cv = shade::d2::TexturedBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.shader = Some(self.shader());
		cv.uniform.transform = Transform2::ortho(viewport.cast());
		cv.uniform.texture = self.texture();

		let screen_center = viewport.cast::<f32>().center();
		let half = self.image_size * 0.5;
		let rotate = Transform2::rotation(self.rotation).around(screen_center);
		let corners = [
			(Vec2(0.0, 0.0), Vec2(-half.x, -half.y)),
			(Vec2(0.0, 1.0), Vec2(-half.x,  half.y)),
			(Vec2(1.0, 1.0), Vec2( half.x,  half.y)),
			(Vec2(1.0, 0.0), Vec2( half.x, -half.y)),
		];

		{
			let mut prim = cv.begin(shade::PrimType::Triangles, 4, 2);
			for &(uv, local) in &corners {
				let scaled = (self.pan + local) * self.zoom;
				let pos = rotate * (screen_center + scaled);
				let color = Vec4::dup(255);
				prim.add_vertex(shade::d2::TexturedVertex { pos, uv, color });
			}
			prim.add_indices_quad();
		}

		cv.draw(g);
		g.end();
	}

	fn draw_hud(&self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let mut hud = d2::TextBuffer::new();
		hud.blend_mode = shade::BlendMode::Alpha;
		hud.uniform.transform = Transform2::ortho(viewport.cast());
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
			concat!(
				"image: {}\n",
				"mode: {}\n",
				"post fx: {}\n",
				"zoom: {:.2}x\n",
				"rotation: {:.1} deg\n",
				"1 = nearest  2 = linear  3 = pixel art\n",
				"P = cycle post fx\n",
				"left drag = pan\n",
				"right drag = rotate + zoom\n",
				"wheel = zoom\n",
				"F2 = open image",
			),
			self.image_name,
			self.filter_mode.label(),
			match self.post_process_mode {
				PostProcessMode::None => "none",
				PostProcessMode::Crt => "crt",
			},
			self.zoom,
			self.rotation.to_deg(),
		);
		hud.text_write(&self.hud_font, &mut scribe, &mut pos, &hud_text);
		hud.draw(g);
	}
}

impl DemoInterface for PixelArt {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::Continuous
	}

	fn resize(&mut self, size: Vec2i) {
		self.viewport = Bounds2!(0, 0, size.x, size.y);
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position } => {
				let delta = position - self.cursor;
				self.cursor = position;
				self.apply_drag_delta(delta);
				self.update_turn_zoom();
				shell.request_redraw();
			}
			Input::MouseButton { button, pressed, position } => {
				self.cursor = position;
				match (button, pressed) {
					(MouseButton::Left, true) => self.drag_mode = DragMode::Pan,
					(MouseButton::Right, true) => self.begin_turn_zoom(self.viewport),
					(MouseButton::Left | MouseButton::Right, false) => self.drag_mode = DragMode::None,
					_ => {}
				}
				shell.request_redraw();
			}
			Input::MouseWheel { delta, position } => {
				self.cursor = position;
				self.apply_zoom_delta(-delta.y * 0.0025);
				shell.request_redraw();
			}
			Input::KeyDown(Key::Digit1) => self.filter_mode = FilterMode::Nearest,
			Input::KeyDown(Key::Digit2) => self.filter_mode = FilterMode::Linear,
			Input::KeyDown(Key::Digit3) => self.filter_mode = FilterMode::PixelArt,
			Input::KeyDown(Key::P) => self.post_process_mode = self.post_process_mode.next(),
			Input::KeyDown(Key::F2) => shell.open_file(FileRequest {
				id: OPEN_IMAGE_REQUEST,
				title: "Open pixel art image",
				extensions: &["png", "gif", "jpg", "jpeg"],
			}),
			_ => {}
		}
	}

	fn file_opened(&mut self, request_id: u32, path: Option<String>, bytes: Option<Vec<u8>>, g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		if request_id == OPEN_IMAGE_REQUEST {
			if let Some(bytes) = bytes {
				if let Err(err) = self.load_image_bytes(g, path, &bytes) {
					shell.set_status(&err);
				}
			}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		self.viewport = viewport;
		self.draw_scene(g, viewport);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.0, 0.0, 0.0, 1.0));

		let texture = &**self.render_texture.as_ref().expect("render texture was not created");
		match self.post_process_mode {
			PostProcessMode::None => self.pp.draw(g,
				&*self.pp_copy_shader,
				shade::BlendMode::Solid,
				&[&shade::shaders::PostProcessCopyUniforms {
					texture,
				}],
			),
			PostProcessMode::Crt => self.pp.draw(g,
				&*self.pp_crt_shader,
				shade::BlendMode::Solid,
				&[&shade::shaders::PostProcessCrtUniforms {
					texture,
					scanline_count: viewport.height() as f32 * 0.25,
					time: frame.time as f32,
					..Default::default()
				}],
			),
		}

		self.draw_turn_zoom_overlay(g, viewport);
		self.draw_hud(g, viewport);
		g.end();
	}
}
