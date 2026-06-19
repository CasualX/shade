use crate::*;

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Text3d::new(g, assets))
}

struct Text3d {
	font: d2::FontResource<shade::msdfgen::Font>,
	camera: d3::ArcballCamera,
	axes: d3::axes::AxesModel,
	color3d_shader: Box<dyn shade::ShaderProgram>,
	cursor: Vec2f,
	left_drag: bool,
	right_drag: bool,
}

impl Text3d {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Text3d {
		let mut shader_source = shade::shader_interface! {
			files {
				"mtsdf.glsl" => shade::shaders::MTSDF,
				"color3d.glsl" => shade::shaders::COLOR3D,
			}
		};
		let font = load_font(g, assets, true);
		let color3d_shader = g.shader_compile(&mut shader_source, "color3d.glsl", &[]);
		let axes = d3::axes::AxesModel::create(g);
		let camera = d3::ArcballCamera::new(Vec3f(0.0, -8.0, 4.6), Vec3f(0.0, -0.4, 0.9), Vec3f::Z);
		Text3d { font, camera, axes, color3d_shader, cursor: Vec2f::ZERO, left_drag: false, right_drag: false }
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
		d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
	}

	fn draw_text(&self, g: &mut shade::Graphics, camera: &d3::Camera) {
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

		let planes = [
			(
				"front plane",
				Vec4(255, 255, 255, 255),
				Vec4(13, 24, 36, 255),
				Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(-2.7, 0.6, 1.6)),
				52.0,
				0.018,
			),
			(
				"on the floor",
				Vec4(255, 220, 130, 255),
				Vec4(20, 14, 4, 255),
				Transform3f::compose(Vec3f::X, Vec3f::Y, Vec3f::Z, Vec3f(-2.8, -2.3, 0.0)),
				48.0,
				0.018,
			),
			(
				"side wall",
				Vec4(140, 220, 255, 255),
				Vec4(4, 18, 28, 255),
				Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(-3.0, -1.4, 0.9)),
				44.0,
				0.017,
			),
			(
				"tilted placard",
				Vec4(180, 255, 185, 255),
				Vec4(2, 26, 11, 255),
				Transform3f::translation(Vec3f(1.2, -0.6, 1.4)) * Transform3f::rotation(Vec3f::Z, Angle::deg(-28.0)) * Transform3f::rotation(Vec3f::X, Angle::deg(62.0)),
				46.0,
				0.017,
			),
			(
				"↑↓←→↔↕\n★☆✓✗●○\n▴▾◂▸\n▲▼◀▶",
				Vec4(255, 240, 120, 255),
				Vec4(38, 26, 0, 255),
				Transform3f::compose(Vec3f::Y, Vec3f::Z, Vec3f::X, Vec3f(3.2, -0.5, 1.5)),
				32.0,
				0.012,
			),
			(
				"[\x1b[draw_mask=false]#\x1b[draw_mask=true]] Emptyness\n[#] Fullness\n☐☑☒  🗹🗷",
				Vec4(210, 225, 255, 255),
				Vec4(10, 20, 42, 255),
				Transform3f::compose(Vec3f::X, Vec3f::Z, -Vec3f::Y, Vec3f(0.5, 2.8, 1.5)),
				28.0,
				0.013,
			),
			(
				"\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W\n⏰💎🔹⚡⛔🏁",
				Vec4(255, 255, 255, 255),
				Vec4(28, 10, 16, 255),
				Transform3f::translation(Vec3f(2.6, 1.0, 2.4)) * Transform3f::rotation(Vec3f::Y, Angle::deg(-25.0)) * Transform3f::rotation(Vec3f::X, Angle::deg(20.0)),
				30.0,
				0.014,
			),
		];
		for (text, color, outline, plane, font_size, scale) in planes {
			scribe.font_size = font_size;
			scribe.line_height = font_size * 1.2;
			scribe.color = color;
			scribe.outline = outline;
			scribe.set_baseline_relative(0.5);
			self.add_text_plane(g, &mut buf, text, scale, plane, &scribe);
		}
	}

	fn add_text_plane<'a>(&'a self, g: &mut shade::Graphics, buf: &mut d2::TextBuffer3<'a>, text: &str, scale: f32, plane: Transform3f, scribe: &d2::Scribe) {
		buf.uniform.plane_transform = plane;
		buf.uniform.text.transform = Transform2f::compose(Vec2f(scale, 0.0), Vec2f(0.0, -scale), Vec2f::ZERO);
		let bounds = Bounds2f::point(Vec2f::ZERO, Vec2f::ZERO);
		buf.text_box(&self.font, scribe, &bounds, d2::TextAlign::MiddleCenter, text);
		buf.draw(g);
	}
}

impl DemoInterface for Text3d {
	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseButton { button: MouseButton::Left, pressed, .. } => self.left_drag = pressed,
			Input::MouseButton { button: MouseButton::Right, pressed, .. } => self.right_drag = pressed,
			Input::MouseMove { position } => {
				let delta = position - self.cursor;
				self.cursor = position;
				if self.left_drag {
					self.camera.rotate(-delta.x, -delta.y);
					shell.request_redraw();
				}
				if self.right_drag {
					self.camera.zoom(delta.y * 0.01);
					shell.request_redraw();
				}
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.08, 0.10, 0.12, 1.0), depth: 1.0);
		let camera = self.camera(frame.viewport);
		self.axes.draw(g, &*self.color3d_shader, &camera, &d3::axes::AxesInstance {
			local: Transform3f::scaling(Vec3f::dup(3.0)),
			depth_test: Some(shade::Compare::LessEqual),
		});
		self.draw_text(g, &camera);
		g.end();
	}
}
