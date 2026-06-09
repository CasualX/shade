use shade::cvmath::*;
use shade::{d2, d3};

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	font: d2::FontResource<shade::msdfgen::Font>,
	camera: d3::ArcballCamera,
	axes: d3::axes::AxesModel,
	left_click: bool,
	middle_click: bool,
	right_click: bool,
}

impl Context {
	pub fn new() -> Context {
		shade::webgl::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		});
		let g = webgl.as_graphics();
		let mut shader_source = shade::shader_interface! {
			files {
				"mtsdf.glsl" => include_str!("../../../src/shaders/mtsdf.glsl"),
				"color3d.glsl" => include_str!("../../../src/shaders/color3d.glsl"),
			}
		};

		let font = {
			let font: shade::msdfgen::FontDto =
				serde_json::from_str(include_str!("../../font/font.json")).unwrap();
			let font: shade::msdfgen::Font = font.into();
			let texture = {
				let file_png = include_bytes!("../../font/font.png");
				let image = shade::image::ImageRGBA::load_memory_png(file_png)
					.unwrap()
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
			let shader = g.shader_compile(&mut shader_source, "mtsdf.glsl", &[shade::ShaderDefine {
				name: "MTSDF_3D",
				value: None,
			}]);
			d2::FontResource {
				font,
				texture,
				shader,
			}
		};

		let color3d_shader = g.shader_compile(&mut shader_source, "color3d.glsl", &[]);
		let axes = d3::axes::AxesModel::create(g, color3d_shader);
		let camera = d3::ArcballCamera::new(Vec3f(0.0, -8.0, 4.6), Vec3f(0.0, -0.4, 0.9), Vec3f::Z);

		Context {
			webgl,
			screen_size: Vec2::ZERO,
			font,
			camera,
			axes,
			left_click: false,
			middle_click: false,
			right_click: false,
		}
	}

	fn camera(&self, viewport: Bounds2i) -> d3::Camera {
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let position = self.camera.position();
		let hand = Hand::RH;
		let view = self.camera.view_matrix(hand);
		let clip = Clip::NO;
		let (near, far) = (0.01, 200.0);
		let projection = Mat4::perspective(Angle::deg(55.0), aspect_ratio, near, far, (hand, clip));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		d3::Camera {
			viewport,
			aspect_ratio,
			position,
			near,
			far,
			view,
			projection,
			view_proj,
			inv_view_proj,
			clip,
		}
	}

	fn draw_text(
		font: &d2::FontResource<shade::msdfgen::Font>,
		g: &mut shade::Graphics,
		camera: &d3::Camera,
	) {
		let mut buf = d2::TextBuffer3::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.depth_test = Some(shade::Compare::LessEqual);
		buf.cull_mode = None;
		buf.uniform.camera_transform = camera.view_proj;
		buf.uniform.text.outline_width_relative = 0.10;

		let mut scribe = d2::Scribe {
			font_width_scale: 1.0,
			letter_spacing: -1.5,
			..Default::default()
		};

		scribe.font_size = 52.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(255, 255, 255, 255);
		scribe.outline = Vec4(13, 24, 36, 255);
		scribe.set_baseline_relative(0.5);
		let front = Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(-2.7, 0.6, 1.6));
		Self::add_text_plane(font, g, &mut buf, "front plane", 0.018, front, &scribe);

		scribe.font_size = 48.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(255, 220, 130, 255);
		scribe.outline = Vec4(20, 14, 4, 255);
		scribe.set_baseline_relative(0.5);
		let floor = Transform3f::compose(Vec3f::X, Vec3f::Y, Vec3f::Z, Vec3f(-2.8, -2.3, 0.0));
		Self::add_text_plane(font, g, &mut buf, "on the floor", 0.018, floor, &scribe);

		scribe.font_size = 44.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(140, 220, 255, 255);
		scribe.outline = Vec4(4, 18, 28, 255);
		scribe.set_baseline_relative(0.5);
		let wall = Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(-3.0, -1.4, 0.9));
		Self::add_text_plane(font, g, &mut buf, "side wall", 0.017, wall, &scribe);

		scribe.font_size = 46.0;
		scribe.line_height = scribe.font_size * 1.2;
		scribe.color = Vec4(180, 255, 185, 255);
		scribe.outline = Vec4(2, 26, 11, 255);
		scribe.set_baseline_relative(0.5);
		let tilted = Transform3f::translation(Vec3f(1.2, -0.6, 1.4))
			* Transform3f::rotation(Vec3f::Z, Angle::deg(-28.0))
			* Transform3f::rotation(Vec3f::X, Angle::deg(62.0));
		Self::add_text_plane(font, g, &mut buf, "tilted placard", 0.017, tilted, &scribe);

		scribe.font_size = 32.0;
		scribe.line_height = 36.0;
		scribe.color = Vec4(255, 240, 120, 255);
		scribe.outline = Vec4(38, 26, 0, 255);
		scribe.top_skew = 0.0;
		scribe.set_baseline_relative(0.5);
		let glyph_wall = Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(3.2, -0.5, 1.5));
		Self::add_text_plane(font, g, &mut buf, "↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶", 0.012, glyph_wall, &scribe);

		scribe.font_size = 28.0;
		scribe.line_height = 31.0;
		scribe.color = Vec4(210, 225, 255, 255);
		scribe.outline = Vec4(10, 20, 42, 255);
		scribe.set_baseline_relative(0.5);
		let status_board = Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(0.5, 2.8, 1.5));
		let text = "[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness\n☐☑☒  🗹🗷";
		Self::add_text_plane(font, g, &mut buf, text, 0.013, status_board, &scribe);

		scribe.font_size = 30.0;
		scribe.line_height = 34.0;
		scribe.color = Vec4(255, 255, 255, 255);
		scribe.outline = Vec4(28, 10, 16, 255);
		scribe.top_skew = 8.0;
		scribe.set_baseline_relative(0.5);
		let rainbow = Transform3f::translation(Vec3f(2.6, 1.0, 2.4))
			* Transform3f::rotation(Vec3f::Y, Angle::deg(-25.0))
			* Transform3f::rotation(Vec3f::X, Angle::deg(20.0));
		let text = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W\n⏰💎🔹⚡⛔🏁";
		Self::add_text_plane(font, g, &mut buf, text, 0.014, rainbow, &scribe);

		scribe.font_size = 26.0;
		scribe.line_height = 29.0;
		scribe.top_skew = 0.0;
		scribe.color = Vec4(190, 255, 220, 255);
		scribe.outline = Vec4(5, 32, 18, 255);
		scribe.set_baseline_relative(0.5);
		let corner_notes = Transform3f::translation(Vec3f(-0.4, -3.0, 1.2))
			* Transform3f::rotation(Vec3f::X, Angle::deg(86.0))
			* Transform3f::rotation(Vec3f::Z, Angle::deg(14.0));
		let text = "△▽◁▷\ntext in orbit\nshade glyph picnic";
		Self::add_text_plane(font, g, &mut buf, text, 0.012, corner_notes, &scribe);
	}

	fn add_text_plane(
		font: &d2::FontResource<shade::msdfgen::Font>,
		g: &mut shade::Graphics,
		buf: &mut d2::TextBuffer3,
		text: &str,
		scale: f32,
		plane: Transform3f,
		scribe: &d2::Scribe,
	) {
		buf.uniform.plane_transform = plane;
		buf.uniform.text.transform =
			Transform2f::compose(Vec2f(scale, 0.0), Vec2f(0.0, -scale), Vec2f::ZERO);

		let bounds = Bounds2f::point(Vec2f::ZERO, Vec2f::ZERO);
		buf.text_box(font, scribe, &bounds, d2::TextAlign::MiddleCenter, text);
		buf.draw(g);
	}
}

impl crate::DemoContext for Context {
	fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2(width, height);
	}

	fn mousemove(&mut self, dx: f32, dy: f32) {
		if self.left_click {
			self.camera.rotate(-dx, -dy);
		}
		if self.middle_click || self.right_click {
			self.camera.zoom(dy * 0.01);
		}
	}

	fn mousedown(&mut self, button: u32) {
		match button {
			0 => self.left_click = true,
			1 => self.middle_click = true,
			2 => self.right_click = true,
			_ => {}
		}
	}

	fn mouseup(&mut self, button: u32) {
		match button {
			0 => self.left_click = false,
			1 => self.middle_click = false,
			2 => self.right_click = false,
			_ => {}
		}
	}

	fn wheel(&mut self, delta_y: f32) {
		self.camera.zoom(delta_y * 0.002);
	}

	fn draw(&mut self, _time: f64) {
		let viewport = Bounds2!(0, 0, self.screen_size.x, self.screen_size.y);
		let camera = self.camera(viewport);
		let font = &self.font;
		let g = self.webgl.as_graphics();

		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.08, 0.10, 0.12, 1.0), depth: 1.0);

		self.axes.draw(g, &camera, &d3::axes::AxesInstance {
			local: Transform3f::scaling(Vec3f::dup(3.0)),
			depth_test: Some(shade::Compare::LessEqual),
		});
		Self::draw_text(font, g, &camera);
		g.end();
	}
}
