use shade::d2;
use shade::cvmath::*;

mod api;

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	font: d2::FontResource<shade::msdfgen::Font>,
}

impl Context {
	pub fn new() -> Context {
		shade::webgl::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		});
		let g = webgl.as_graphics();

		let font = {
			// Parse the font metadata
			let font: shade::msdfgen::FontDto = serde_json::from_str(include_str!("../../../font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();

			// Load the texture
			let texture = {
				let file_png = include_bytes!("../../../font/font.png");
				let image = shade::image::ImageRGBA::load_memory_png(file_png).unwrap().map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
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

			// Compile the shader
			let shader = g.shader_compile(shade::webgl::shaders::MTSDF_VS, shade::webgl::shaders::MTSDF_FS);

			d2::FontResource { font, texture, shader }
		};

		let screen_size = Vec2::ZERO;
		Context { webgl, screen_size, font }
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	pub fn draw(&mut self, _time: f64) {
		let g = self.webgl.as_graphics();
		let size = self.screen_size;
		let font = &self.font;

		// Render the frame
		let viewport = Bounds2::c(0, 0, size.x, size.y);
		g.begin(&shade::BeginArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.4, 0.4, 0.7, 1.0), depth: 1.0);

		let mut cv = d2::TextBuffer::new();
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.uniform.transform = Transform2::ortho(Bounds2::c(0.0, 0.0, size.x as f32, size.y as f32));
		cv.uniform.outline_width_relative = 0.125;

		let mut pos = Vec2(0.0, 0.0);
		let mut scribe = d2::Scribe {
			font_size: 64.0,
			line_height: 64.0 * 1.5,
			x_pos: pos.x,
			top_skew: 8.0,
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);

		cv.text_write(&font, &mut scribe, &mut pos, "Hello, \x1b[font_size=96.0]\x1b[font_width_scale=1.5]\x1b[top_skew=0.0]world!");

		scribe.font_size = 32.0;
		scribe.line_height = 32.0;
		scribe.font_width_scale = 1.0;
		scribe.color = Vec4(255, 255, 0, 255);
		cv.text_box(&font, &scribe, &Bounds2::c(0.0, 0.0, size.x as f32, size.y as f32), d2::TextAlign::MiddleCenter, "These\nare\nmultiple\nlines.\n");

		scribe.top_skew = 8.0;
		let rainbow = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W";
		let rainbow_width = scribe.text_width(&mut {Vec2::ZERO}, &font.font, rainbow);
		let mut pos = Vec2f((size.x as f32 - rainbow_width) * 0.5, size.y as f32 - scribe.font_size);
		cv.text_write(&font, &mut scribe, &mut pos, rainbow);

		cv.draw(g);

		// Finish rendering
		g.end();
	}
}
