use shade::cvmath::*;
use shade::{d2, d3};

const LOOP_DURATION: f32 = 26.0;
const CRAWL_START: f32 = 7.0;
const CRAWL_END: f32 = LOOP_DURATION;
const CRAWL_DISTANCE: f32 = 10.0;
const CRAWL_ANGLE_DEG: f32 = 18.0;
const OPENING_CARD: &str = "OUT BEYOND THE VISIBLE FRUSTUM,\nA NEW RENDER PASSES SILENTLY.";
const TITLE_TEXT: &str = "SHADE\nTEXT INTRO";
const CRAWL_TEXT: &str = "SEQUENCE ONE\nTHE OPENING CRAWL\n\nThe render deck is restless.\nAcross the quiet span of the\nframe, a patient convoy of\nglyphs climbs toward the\nvanishing point, determined\nto look dramatic at any scale.\n\nArmed with signed distance\nfields, careful blending, and\na suspicious amount of golden\noutline, the crew presses on\nthrough perspective and depth.\n\nTheir mission is simple:\ndelight the eye, test the atlas,\nand make the viewport feel\njust a little more heroic.\n\nIf the sampler stays honest\nand the timing holds together,\nthis humble scene might earn\nitself another triumphant loop.";

#[derive(Clone, Copy)]
enum Ease {
	Linear,
	Smooth,
}

impl Ease {
	fn apply(self, t: f32) -> f32 {
		let t = t.clamp(0.0, 1.0);
		match self {
			Ease::Linear => t,
			Ease::Smooth => t * t * (3.0 - 2.0 * t),
		}
	}
}

#[derive(Clone, Copy)]
struct TextProperties {
	position: Vec3f,
	rotation_axis: Vec3f,
	rotation_deg: f32,
	scale: f32,
	alpha: f32,
	font_size: f32,
	line_height: f32,
	letter_spacing: f32,
	color: Vec4<u8>,
	outline: Vec4<u8>,
}

impl TextProperties {
	fn plane_transform(&self) -> Transform3f {
		Transform3f::translation(self.position)
			* Transform3f::rotation(self.rotation_axis, Angle::deg(self.rotation_deg))
	}

	fn scribe(&self) -> d2::Scribe {
		let mut scribe = d2::Scribe {
			font_size: self.font_size,
			line_height: self.line_height,
			font_width_scale: 1.0,
			letter_spacing: self.letter_spacing,
			color: Self::with_alpha(self.color, self.alpha),
			outline: Self::with_alpha(self.outline, self.alpha),
			..Default::default()
		};
		scribe.set_baseline_relative(0.5);
		scribe
	}

	fn with_alpha(mut color: Vec4<u8>, alpha: f32) -> Vec4<u8> {
		color.w = ((color.w as f32) * alpha.clamp(0.0, 1.0)).round() as u8;
		color
	}
}

enum Animation {
	AlphaWindow {
		start: f32,
		end: f32,
		fade: f32,
	},
	MoveAbs {
		from: Vec3f,
		to: Vec3f,
		start: f32,
		end: f32,
		ease: Ease,
	},
	MoveVelocity {
		velocity: Vec3f,
		start: f32,
		end: f32,
	},
	Scale {
		from: f32,
		to: f32,
		start: f32,
		end: f32,
		ease: Ease,
	},
}

impl Animation {
	fn apply(&self, properties: &mut TextProperties, time: f32) {
		match *self {
			Animation::AlphaWindow { start, end, fade } => {
				if time < start || time > end {
					properties.alpha = 0.0;
				}
				else {
					let fade_in = ((time - start) / fade).clamp(0.0, 1.0);
					let fade_out = ((end - time) / fade).clamp(0.0, 1.0);
					properties.alpha = fade_in.min(fade_out);
				}
			}
			Animation::MoveAbs {
				from,
				to,
				start,
				end,
				ease,
			} => {
				let t = Self::progress(start, end, ease, time);
				properties.position = from.lerp(to, t);
			}
			Animation::MoveVelocity {
				velocity,
				start,
				end,
			} => {
				let elapsed = (time.min(end) - start).max(0.0);
				properties.position += velocity * elapsed;
			}
			Animation::Scale {
				from,
				to,
				start,
				end,
				ease,
			} => {
				properties.scale = Self::animate_scalar(from, to, start, end, ease, time);
			}
		}
	}

	fn animate_scalar(from: f32, to: f32, start: f32, end: f32, ease: Ease, time: f32) -> f32 {
		from.lerp(to, Self::progress(start, end, ease, time))
	}

	fn progress(start: f32, end: f32, ease: Ease, time: f32) -> f32 {
		if end <= start {
			return if time >= end { 1.0 } else { 0.0 };
		}
		ease.apply((time - start) / (end - start))
	}
}

struct AnimatedText {
	text: &'static str,
	properties: TextProperties,
	animations: Vec<Animation>,
}

impl AnimatedText {
	fn properties_at(&self, time: f32) -> TextProperties {
		let mut properties = self.properties;
		for animation in &self.animations {
			animation.apply(&mut properties, time);
		}
		properties
	}
}

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: Vec2i,
	font: d2::FontResource<shade::msdfgen::Font>,
	storyboard: Vec<AnimatedText>,
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

		Context {
			webgl,
			screen_size: Vec2::ZERO,
			font,
			storyboard: Self::storyboard(),
		}
	}

	fn camera(&self, viewport: Bounds2i) -> d3::Camera {
		let aspect_ratio = viewport.width() as f32 / viewport.height() as f32;
		let position = Vec3f(0.0, -8.8, 4.2);
		let hand = Hand::RH;
		let view = Transform3f::look_at(position, Vec3f(0.0, 2.4, 0.8), Vec3f::Z, hand);
		let clip = Clip::NO;
		let (near, far) = (0.01, 200.0);
		let projection = Mat4::perspective(Angle::deg(52.0), aspect_ratio, near, far, (hand, clip));
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

	fn storyboard() -> Vec<AnimatedText> {
		let mut storyboard = Vec::new();
		let crawl_velocity = Self::crawl_text_up() * (CRAWL_DISTANCE / (CRAWL_END - CRAWL_START));

		for (pos, scale, color, glyph) in [
			(Vec3f(-4.6, 8.0, 5.8), 0.008, Vec4(255, 255, 255, 180), "*"),
			(Vec3f(-2.4, 6.8, 4.6), 0.006, Vec4(180, 215, 255, 150), "o"),
			(Vec3f(-0.2, 9.1, 6.0), 0.007, Vec4(255, 240, 180, 170), "*"),
			(Vec3f(1.1, 7.1, 5.0), 0.005, Vec4(220, 235, 255, 140), "."),
			(Vec3f(3.5, 8.4, 4.8), 0.006, Vec4(255, 255, 255, 155), "*"),
			(Vec3f(4.7, 6.6, 6.2), 0.005, Vec4(180, 220, 255, 120), "o"),
		] {
			storyboard.push(AnimatedText {
				text: glyph,
				properties: TextProperties {
					position: pos,
					rotation_axis: Vec3f::X,
					rotation_deg: 90.0,
					scale,
					alpha: 1.0,
					font_size: 28.0,
					line_height: 28.0,
					letter_spacing: -1.5,
					color,
					outline: Vec4(0, 0, 0, 0),
				},
				animations: Vec::new(),
			});
		}

		storyboard.push(AnimatedText {
			text: OPENING_CARD,
			properties: TextProperties {
				position: Vec3f(0.0, 0.4, 1.4),
				rotation_axis: Vec3f::X,
				rotation_deg: 90.0,
				scale: 0.014,
				alpha: 1.0,
				font_size: 44.0,
				line_height: 44.0 * 1.15,
				letter_spacing: -1.5,
				color: Vec4(120, 210, 255, 255),
				outline: Vec4(4, 16, 32, 255),
			},
			animations: vec![Animation::AlphaWindow {
				start: 0.0,
				end: 4.8,
				fade: 1.0,
			}],
		});

		storyboard.push(AnimatedText {
			text: TITLE_TEXT,
			properties: TextProperties {
				position: Vec3f(0.0, -0.4, 1.0),
				rotation_axis: Vec3f::X,
				rotation_deg: 90.0,
				scale: 0.056,
				alpha: 1.0,
				font_size: 118.0,
				line_height: 94.0,
				letter_spacing: -4.0,
				color: Vec4(255, 214, 92, 255),
				outline: Vec4(64, 38, 0, 255),
			},
			animations: vec![
				Animation::AlphaWindow {
					start: 3.0,
					end: 9.5,
					fade: 0.9,
				},
				Animation::MoveAbs {
					from: Vec3f(0.0, -0.4, 1.0),
					to: Vec3f(0.0, 4.0, 2.2),
					start: 3.0,
					end: 9.5,
					ease: Ease::Linear,
				},
				Animation::Scale {
					from: 0.056,
					to: 0.036,
					start: 3.0,
					end: 9.5,
					ease: Ease::Smooth,
				},
			],
		});

		storyboard.push(AnimatedText {
			text: CRAWL_TEXT,
			properties: TextProperties {
				position: Vec3f(0.0, -6.8, -2.0),
				rotation_axis: Vec3f::X,
				rotation_deg: CRAWL_ANGLE_DEG,
				scale: 0.013,
				alpha: 1.0,
				font_size: 32.0,
				line_height: 36.0,
				letter_spacing: -0.8,
				color: Vec4(255, 212, 96, 255),
				outline: Vec4(52, 28, 0, 255),
			},
			animations: vec![
				Animation::AlphaWindow {
					start: CRAWL_START,
					end: LOOP_DURATION,
					fade: 1.4,
				},
				Animation::MoveVelocity {
					velocity: crawl_velocity,
					start: CRAWL_START,
					end: CRAWL_END,
				},
			],
		});

		storyboard.push(AnimatedText {
			text: "looping forever in the viewport",
			properties: TextProperties {
				position: Vec3f(0.0, 4.8, 2.0),
				rotation_axis: Vec3f::X,
				rotation_deg: 90.0,
				scale: 0.012,
				alpha: 1.0,
				font_size: 24.0,
				line_height: 28.0,
				letter_spacing: -1.5,
				color: Vec4(180, 220, 255, 255),
				outline: Vec4(8, 18, 30, 255),
			},
			animations: vec![Animation::AlphaWindow {
				start: 22.0,
				end: LOOP_DURATION,
				fade: 0.8,
			}],
		});

		storyboard
	}

	fn crawl_text_up() -> Vec3f {
		let angle = CRAWL_ANGLE_DEG.to_radians();
		Vec3f(0.0, angle.cos(), angle.sin())
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

	fn draw(&mut self, time: f64) {
		let size = self.screen_size;
		let viewport = Bounds2!(0, 0, size.x, size.y);
		let elapsed = (time as f32) % LOOP_DURATION;
		let camera = self.camera(viewport);
		let font = &self.font;
		let g = self.webgl.as_graphics();

		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.01, 0.01, 0.03, 1.0), depth: 1.0);

		let mut buf = d2::TextBuffer3::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.depth_test = Some(shade::Compare::LessEqual);
		buf.cull_mode = None;
		buf.uniform.camera_transform = camera.view_proj;
		buf.uniform.text.outline_width_relative = 0.10;

		for item in &self.storyboard {
			let properties = item.properties_at(elapsed);
			if properties.alpha <= 0.0 {
				continue;
			}
			let scribe = properties.scribe();
			Self::add_text_plane(
				font,
				g,
				&mut buf,
				item.text,
				properties.scale,
				properties.plane_transform(),
				&scribe,
			);
		}

		g.end();
	}
}
