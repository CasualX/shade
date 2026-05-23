use shade::cvmath::*;
use shade::d2;

mod api;

const DEFAULT_IMAGE_NAME: &str = "lapras.png";
const DEFAULT_IMAGE_BYTES: &[u8] = include_bytes!("../../../textures/lapras.png");

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
enum DragMode {
	None,
	Pan,
	TurnZoom,
}

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	textured_shader: shade::ShaderProgram,
	pixelart_shader: shade::ShaderProgram,
	hud_font: d2::FontResource<shade::msdfgen::Font>,
	nearest_texture: shade::Texture2D,
	linear_texture: shade::Texture2D,
	image_size: Vec2f,
	filter_mode: FilterMode,
	pan: Vec2f,
	zoom: f32,
	rotation: f32,
	drag_mode: DragMode,
}

impl Context {
	pub fn new() -> Context {
		shade::webgl::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		});
		let g = webgl.as_graphics();

		let textured_shader = g.shader_compile(
			shade::shaders::glsl300es::TEXTURED_VS,
			shade::shaders::glsl300es::TEXTURED_FS,
		);
		let pixelart_shader = g.shader_compile(
			shade::shaders::glsl300es::PIXELART_VS,
			shade::shaders::glsl300es::PIXELART_FS,
		);
		let hud_font = {
			let font: shade::msdfgen::FontDto = serde_json::from_str(include_str!("../../../font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();
			let texture = {
				let file_png = include_bytes!("../../../font/font.png");
				let image = shade::image::ImageRGBA::load_memory_png(file_png).unwrap()
					.map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
				let props = shade::TextureProps {
					mip_levels: 1,
					usage: shade::TextureUsage::TEXTURE,
					filter_min: shade::TextureFilter::Linear,
					filter_mag: shade::TextureFilter::Linear,
					wrap_u: shade::TextureWrap::Edge,
					wrap_v: shade::TextureWrap::Edge,
					..Default::default()
				};
				g.image(&(&image, &props))
			};
			let shader = g.shader_compile(shade::shaders::glsl300es::MTSDF_VS, shade::shaders::glsl300es::MTSDF_FS);
			d2::FontResource { font, texture, shader }
		};

		let image = shade::image::DecodedImage::load_memory_png(DEFAULT_IMAGE_BYTES).unwrap();
		let image_size = Vec2(image.width() as f32, image.height() as f32);
		let nearest_props = shade::TextureProps {
			mip_levels: 1,
			usage: shade::TextureUsage::TEXTURE,
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			wrap_u: shade::TextureWrap::Edge,
			wrap_v: shade::TextureWrap::Edge,
			..Default::default()
		};
		let linear_props = shade::TextureProps {
			mip_levels: 1,
			usage: shade::TextureUsage::TEXTURE,
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Edge,
			wrap_v: shade::TextureWrap::Edge,
			..Default::default()
		};
		let nearest_texture = g.image(&(&image, &nearest_props));
		let linear_texture = g.image(&(&image, &linear_props));

		Context {
			webgl,
			screen_size: Vec2::ZERO,
			textured_shader,
			pixelart_shader,
			hud_font,
			nearest_texture,
			linear_texture,
			image_size,
			filter_mode: FilterMode::PixelArt,
			pan: Vec2::ZERO,
			zoom: 1.0,
			rotation: 0.0,
			drag_mode: DragMode::None,
		}
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn mousemove(&mut self, dx: f32, dy: f32) {
		let delta = Vec2(dx, dy);
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

	pub fn mousedown(&mut self, button: u32) {
		self.drag_mode = match button {
			0 => DragMode::Pan,
			2 => DragMode::TurnZoom,
			_ => self.drag_mode,
		};
	}

	pub fn mouseup(&mut self, button: u32) {
		if matches!(button, 0 | 2) {
			self.drag_mode = DragMode::None;
		}
	}

	pub fn wheel(&mut self, delta_y: f32) {
		self.apply_zoom_delta(-delta_y * 0.0025);
	}

	pub fn keydown(&mut self, key: u32) {
		self.filter_mode = match key {
			1 => FilterMode::Nearest,
			2 => FilterMode::Linear,
			3 => FilterMode::PixelArt,
			_ => self.filter_mode,
		};
	}

	pub fn draw(&mut self, _time: f64) {
		let size = self.screen_size;
		let shader = self.shader();
		let texture = self.texture();
		let filter_label = self.filter_mode.label();
		let zoom = self.zoom;
		let rotation_degrees = self.rotation.to_degrees();
		let pan = self.pan;
		let image_size = self.image_size;
		let rotation = self.rotation;
		let g = self.webgl.as_graphics();
		let viewport = Bounds2!(0, 0, size.x, size.y);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.08, 0.09, 0.10, 1.0));

		let mut cv = d2::TexturedBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.shader = shader;
		cv.uniform.transform = Transform2::ortho(Bounds2!(0.0, 0.0, size.x as f32, size.y as f32));
		cv.uniform.texture = texture;

		let screen_center = Vec2(size.x as f32 * 0.5, size.y as f32 * 0.5);
		let half = Vec2(image_size.x * 0.5, image_size.y * 0.5);
		let rotate = Transform2::rotation(Angle(rotation)).around(screen_center);
		let corners = [
			(Vec2(0.0, 0.0), Vec2(-half.x, -half.y)),
			(Vec2(0.0, 1.0), Vec2(-half.x, half.y)),
			(Vec2(1.0, 1.0), Vec2(half.x, half.y)),
			(Vec2(1.0, 0.0), Vec2(half.x, -half.y)),
		];

		{
			let mut prim = cv.begin(shade::PrimType::Triangles, 4, 2);
			for &(uv, local) in &corners {
				let scaled = (pan + local) * zoom;
				prim.add_vertex(shade::d2::TexturedVertex {
					pos: rotate * (screen_center + scaled),
					uv,
					color: Vec4::dup(255),
				});
			}
			prim.add_indices_quad();
		}

		cv.draw(g);

		let mut hud = d2::TextBuffer::new();
		hud.blend_mode = shade::BlendMode::Alpha;
		hud.uniform.transform = Transform2::ortho(Bounds2!(0.0, 0.0, size.x as f32, size.y as f32));
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
			"image: {}\nmode: {}\nzoom: {:.2}x\nrotation: {:.1} deg\n1 = nearest  2 = linear  3 = pixel art\nleft drag = pan\nright drag = rotate + zoom\nwheel = zoom",
			DEFAULT_IMAGE_NAME,
			filter_label,
			zoom,
			rotation_degrees,
		);
		hud.text_write(&self.hud_font, &mut scribe, &mut pos, &hud_text);
		hud.draw(g);

		g.end();
	}

	fn apply_zoom_delta(&mut self, delta: f32) {
		self.zoom = (self.zoom * delta.exp()).clamp(1.0 / 64.0, 64.0);
	}

	fn shader(&self) -> shade::ShaderProgram {
		match self.filter_mode {
			FilterMode::Nearest | FilterMode::Linear => self.textured_shader,
			FilterMode::PixelArt => self.pixelart_shader,
		}
	}

	fn texture(&self) -> shade::Texture2D {
		match self.filter_mode {
			FilterMode::Nearest => self.nearest_texture,
			FilterMode::Linear | FilterMode::PixelArt => self.linear_texture,
		}
	}
}