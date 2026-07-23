use std::fmt::Write as _;

use crate::*;

const STREAM_SLOTS: u32 = 56;
const STREAM_CENTER: Vec3f = Vec3f(0.0, 0.0, 0.0);
const MATRIX_SEED: u64 = 0x4D41_5452_4958_5241;
const GLYPHS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz@#$%&*+-=<>?/\\|[]{}()";

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(TextMatrix::new(g, assets))
}

struct RainStream {
	depth: f32,
	position: Vec3f,
	scale: f32,
	font_size: f32,
	alpha: f32,
	glyph_seed: u64,
	length: usize,
	mutation_rate: f32,
	palette_phase: f32,
	pulse_phase: f32,
}

impl RainStream {
	fn line_step(&self) -> f32 {
		self.font_size * self.scale
	}

	fn plane_transform(&self, position: Vec3f, camera_position: Vec3f) -> Transform3f {
		let to_camera = camera_position - position;
		let yaw = Angle::atan2(to_camera.x, -to_camera.y);
		Transform3f::translation(position) * Transform3f::rotation(Vec3f::Z, yaw) * Transform3f::rotation(Vec3f::X, Angle::deg(90.0))
	}

	fn head_position(&self) -> Vec3f {
		self.position - Vec3f::Z * self.line_step() * (self.length - 1) as f32
	}
}

struct TextMatrix {
	font: d2::FontResource<shade::atlas::Font>,
	camera: d3::ArcballCamera,
	cursor: Vec2f,
	left_drag: bool,
}

impl TextMatrix {
	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> TextMatrix {
		let font = load_font(g, assets, "font/font.json", "font/font.png", true);
		let camera = d3::ArcballCamera::new(Vec3f(0.0, -16.85, 0.6), STREAM_CENTER, Vec3f::Z);
		TextMatrix {
			font,
			camera,
			cursor: Vec2f::ZERO,
			left_drag: false,
		}
	}

	fn camera(&self, viewport: Bounds2i, time: f32) -> d3::Camera {
		let aspect_ratio = viewport.width() as f32 / viewport.height().max(1) as f32;
		let dolly = 0.75 - 0.75 * (time * 0.105).cos();
		let mut animated_camera = self.camera.clone();
		animated_camera.radius -= dolly * 0.75;
		let position = animated_camera.position();
		let hand = Hand::RH;
		let view = animated_camera.view_matrix(hand);
		let clip = Clip::NO;
		let (near, far) = (0.01, 80.0);
		let projection = Mat4::perspective(Angle::deg(54.0), aspect_ratio, near, far, (hand, clip));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
	}

	fn streams(&self, time: f32) -> Vec<RainStream> {
		let mut streams = Vec::with_capacity(STREAM_SLOTS as usize);
		let mut matrix_rng = urandom::seeded(MATRIX_SEED);
		for _ in 0..STREAM_SLOTS {
			let mut slot_rng = matrix_rng.split();
			let cycle = slot_rng.range(6.2f32..13.6);
			let offset = slot_rng.range(0.0f32..cycle);
			let stream_time = time + offset;
			let generation = (stream_time / cycle).floor() as u32;
			let age = stream_time % cycle;
			let active_time = cycle - 1.15;
			if age >= active_time {
				continue;
			}

			let mut rng = urandom::seeded(slot_rng.next_u64() ^ generation as u64);
			let depth = rng.range(0.2f32..13.0);
			let x_extent = 4.6 + depth * 0.26;
			let x = rng.range(-x_extent..x_extent);
			let length = rng.range(9usize..24);
			let font_size = rng.range(27.0f32..35.0);
			let scale = rng.range(0.0115f32..0.016);
			let line_step = font_size * scale;
			let speed = rng.range(8.8f32..12.2) / active_time;
			let head_start = rng.range(4.4f32..6.6);
			let z = head_start + line_step * (length - 1) as f32 - speed * age;
			let fade_in = smoothstep(0.0, 0.45, age);
			let fade_out = smoothstep(0.0, 0.9, active_time - age);
			let depth_alpha = 0.48 + (1.0 - depth / 13.0) * 0.52;
			let mutation_rate = rng.range(5.0f32..14.0);
			let palette_phase = rng.range(0.0f32..std::f32::consts::TAU);
			let pulse_phase = rng.range(0.0f32..std::f32::consts::TAU);
			let glyph_seed = rng.next_u64();

			streams.push(RainStream {
				depth,
				position: Vec3f(x, depth - 6.6, z - 3.4),
				scale,
				font_size,
				alpha: fade_in.min(fade_out) * depth_alpha,
				glyph_seed,
				length,
				mutation_rate,
				palette_phase,
				pulse_phase,
			});
		}
		streams
	}

	fn stream_text(&self, stream: &RainStream, time: f32) -> (String, char) {
		let mut text = String::with_capacity(stream.length * 50);
		let mutation_tick = (time * stream.mutation_rate).floor() as u32;
		let palette_wave = 0.5 + 0.5 * (time * 1.7 + stream.palette_phase).sin();
		let near = 1.0 - (stream.depth / 13.0).clamp(0.0, 1.0);
		let mut head = '0';

		for row in 0..stream.length {
			let trail = row as f32 / (stream.length - 1) as f32;
			let row_tick = mutation_tick / (1 + (row as u32 % 4));
			let glyph_seed = stream.glyph_seed ^ ((row as u64) << 32) ^ row_tick as u64;
			let mut glyph_rng = urandom::seeded(glyph_seed);
			let mut glyph = *glyph_rng.choose(GLYPHS).unwrap() as char;
			if row + 1 != stream.length && glyph_rng.chance(1.0 / 19.0) {
				glyph = ' ';
			}
			if row + 1 == stream.length {
				head = glyph;
			}

			let glow = trail.powf(1.45);
			let (r, g, b);
			if row + 1 == stream.length {
				r = (175.0 + palette_wave * 75.0) as u8;
				g = 255;
				b = (200.0 + palette_wave * 55.0) as u8;
			}
			else {
				r = (3.0 + glow * (34.0 + near * 20.0)) as u8;
				g = (58.0 + glow * 184.0 + palette_wave * 10.0) as u8;
				b = (18.0 + glow * (42.0 + (1.0 - near) * 30.0)) as u8;
			};
			let alpha = (255.0 * stream.alpha * (0.08 + 0.92 * glow)) as u8;
			let outline_alpha = (alpha as f32 * (0.32 + glow * 0.22)) as u8;
			let _ = write!(text, "\x1b[color=#{r:02X}{g:02X}{b:02X}{alpha:02X}]\x1b[outline=#001A08{outline_alpha:02X}]{glyph}");
			if row + 1 != stream.length {
				text.push('\n');
			}
		}
		(text, head)
	}

	fn add_stream<'a>(&'a self, buf: &mut d2::TextBuffer3<'a>, stream: &RainStream, text: &str, position: Vec3f, camera_position: Vec3f, scribe: &d2::Scribe) {
		buf.uniform.plane_transform = stream.plane_transform(position, camera_position);
		buf.uniform.text.transform = Transform2f::compose(Vec2f(stream.scale, 0.0), Vec2f(0.0, -stream.scale), Vec2f::ZERO);
		let bounds = Bounds2f::point(Vec2f::ZERO, Vec2f::ZERO);
		buf.text_box(&self.font, scribe, &bounds, d2::TextAlign::TopCenter, text);
	}
}

impl DemoInterface for TextMatrix {
	fn input(&mut self, input: Input, _g: &mut dyn shade::IGraphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseButton {
				button: gui::MouseButton::LEFT,
				pressed,
				position,
			} => {
				self.cursor = position;
				self.left_drag = pressed;
			}
			Input::MouseMove { position } => {
				let delta = position - self.cursor;
				self.cursor = position;
				if self.left_drag {
					self.camera.rotate(-delta.x, -delta.y);
					shell.request_redraw();
				}
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		let time = frame.time as f32;
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.0, 0.008, 0.003, 1.0), depth: 1.0);

		let camera = self.camera(frame.viewport, time);
		let mut streams = self.streams(time);
		streams.sort_by(|a, b| {
			let a_distance = (a.position - camera.position).len_sqr();
			let b_distance = (b.position - camera.position).len_sqr();
			b_distance.total_cmp(&a_distance)
		});
		let mut buf = d2::TextBuffer3::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.depth_test = Some(shade::Compare::LessEqual);
		buf.cull_mode = None;
		buf.uniform.camera_transform = camera.view_proj;
		buf.uniform.text.outline_width_relative = 0.12;

		let mut heads = Vec::with_capacity(streams.len());
		for stream in &streams {
			let (text, head) = self.stream_text(stream, time);
			let mut scribe = d2::Scribe {
				font_size: stream.font_size,
				line_height: stream.font_size,
				letter_spacing: -1.0,
				color: Vec4(0, 255, 70, 255),
				outline: Vec4(0, 18, 5, 128),
				..Default::default()
			};
			scribe.set_baseline_relative(0.5);
			self.add_stream(&mut buf, stream, &text, stream.position, camera.position, &scribe);
			heads.push((stream, head));
		}

		buf.blend_mode = shade::BlendMode::Additive;
		buf.uniform.text.outline_width_relative = 0.24;
		for (stream, head) in heads {
			let pulse = 0.55 + 0.45 * (time * 7.0 + stream.pulse_phase).sin().abs();
			let alpha = (stream.alpha * pulse * 145.0) as u8;
			let mut scribe = d2::Scribe {
				font_size: stream.font_size,
				line_height: stream.font_size,
				letter_spacing: -1.0,
				color: Vec4(190, 255, 220, alpha),
				outline: Vec4(0, 255, 90, alpha / 2),
				..Default::default()
			};
			scribe.set_baseline_relative(0.5);
			self.add_stream(
				&mut buf,
				stream,
				&head.to_string(),
				stream.head_position(),
				camera.position,
				&scribe,
			);
		}

		buf.draw(g);
		g.end();
	}
}
