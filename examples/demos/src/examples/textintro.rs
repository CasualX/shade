use crate::*;

const LOOP_DURATION: f32 = 26.0;
const CRAWL_START: f32 = 7.0;
const CRAWL_END: f32 = LOOP_DURATION;
const CRAWL_DISTANCE: f32 = 10.0;
const CRAWL_ANGLE_DEG: f32 = 18.0;
const OPENING_CARD: &str = "OUT BEYOND THE VISIBLE FRUSTUM,\nA NEW RENDER PASSES SILENTLY.";
const TITLE_TEXT: &str = "SHADE\nTEXT INTRO";
const CRAWL_TEXT: &str = "SEQUENCE ONE\nTHE OPENING CRAWL\n\nThe render deck is restless.\nAcross the quiet span of the\nframe, a patient convoy of\nglyphs climbs toward the\nvanishing point, determined\nto look dramatic at any scale.\n\nArmed with signed distance\nfields, careful blending, and\na suspicious amount of golden\noutline, the crew presses on\nthrough perspective and depth.\n\nTheir mission is simple:\ndelight the eye, test the atlas,\nand make the viewport feel\njust a little more heroic.\n\nIf the sampler stays honest\nand the timing holds together,\nthis humble scene might earn\nitself another triumphant loop.";

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(TextIntro::new(g, assets))
}

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
	rotation: Anglef,
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
		Transform3f::translation(self.position) * Transform3f::rotation(self.rotation_axis, self.rotation)
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
		match self {
			&Animation::AlphaWindow { start, end, fade } => {
				if time < start || time > end {
					properties.alpha = 0.0;
				}
				else {
					properties.alpha = ((time - start) / fade).clamp(0.0, 1.0).min(((end - time) / fade).clamp(0.0, 1.0));
				}
			}
			&Animation::MoveAbs { from, to, start, end, ease } => properties.position = from.lerp(to, Self::progress(start, end, ease, time)),
			&Animation::MoveVelocity { velocity, start, end } => properties.position += velocity * (time.min(end) - start).max(0.0),
			&Animation::Scale { from, to, start, end, ease } => properties.scale = from.lerp(to, Self::progress(start, end, ease, time)),
		}
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

struct TextIntro {
	font: d2::FontResource<shade::atlas::Font>,
	storyboard: Vec<AnimatedText>,
}

impl TextIntro {
	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> TextIntro {
		let font = load_font(g, assets, "font/font.json", "font/font.png", true);
		TextIntro { font, storyboard: Self::storyboard() }
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
		d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
	}

	fn storyboard() -> Vec<AnimatedText> {
		let crawl_velocity = Self::crawl_text_up() * (CRAWL_DISTANCE / (CRAWL_END - CRAWL_START));
		vec![
			AnimatedText {
				text: OPENING_CARD,
				properties: TextProperties {
					position: Vec3f(0.0, 0.4, 1.4),
					rotation_axis: Vec3f::X,
					rotation: Anglef::deg(90.0),
					scale: 0.014,
					alpha: 1.0,
					font_size: 44.0,
					line_height: 44.0 * 1.15,
					letter_spacing: -1.5,
					color: Vec4(120, 210, 255, 255),
					outline: Vec4(4, 16, 32, 255),
				},
				animations: vec![Animation::AlphaWindow { start: 0.0, end: 4.8, fade: 1.0 }],
			},
			AnimatedText {
				text: TITLE_TEXT,
				properties: TextProperties {
					position: Vec3f(0.0, -0.4, 1.0),
					rotation_axis: Vec3f::X,
					rotation: Anglef::deg(90.0),
					scale: 0.056,
					alpha: 1.0,
					font_size: 118.0,
					line_height: 94.0,
					letter_spacing: -4.0,
					color: Vec4(255, 214, 92, 255),
					outline: Vec4(64, 38, 0, 255),
				},
				animations: vec![
					Animation::AlphaWindow { start: 3.0, end: 9.5, fade: 0.9 },
					Animation::MoveAbs { from: Vec3f(0.0, -0.4, 1.0), to: Vec3f(0.0, 4.0, 2.2), start: 3.0, end: 9.5, ease: Ease::Linear },
					Animation::Scale { from: 0.056, to: 0.036, start: 3.0, end: 9.5, ease: Ease::Smooth },
				],
			},
			AnimatedText {
				text: CRAWL_TEXT,
				properties: TextProperties {
					position: Vec3f(0.0, -6.8, -2.0),
					rotation_axis: Vec3f::X,
					rotation: Anglef::deg(CRAWL_ANGLE_DEG),
					scale: 0.013,
					alpha: 1.0,
					font_size: 32.0,
					line_height: 36.0,
					letter_spacing: -0.8,
					color: Vec4(255, 212, 96, 255),
					outline: Vec4(52, 28, 0, 255),
				},
				animations: vec![
					Animation::AlphaWindow { start: CRAWL_START, end: LOOP_DURATION, fade: 1.4 },
					Animation::MoveVelocity { velocity: crawl_velocity, start: CRAWL_START, end: CRAWL_END },
				],
			},
		]
	}

	fn crawl_text_up() -> Vec3f {
		let up = Anglef::deg(CRAWL_ANGLE_DEG).vec2();
		Vec3f(0.0, up.x, up.y)
	}

	fn add_text_plane<'a>(&'a self, g: &mut dyn shade::IGraphics, buf: &mut d2::TextBuffer3<'a>, text: &str, scale: f32, plane: Transform3f, scribe: &d2::Scribe) {
		buf.uniform.plane_transform = plane;
		buf.uniform.text.transform = Transform2f::compose(Vec2f(scale, 0.0), Vec2f(0.0, -scale), Vec2f::ZERO);
		let bounds = Bounds2f::point(Vec2f::ZERO, Vec2f::ZERO);
		buf.text_box(&self.font, scribe, &bounds, d2::TextAlign::MiddleCenter, text);
		buf.draw(g);
	}
}

impl DemoInterface for TextIntro {
	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		let elapsed = frame.time as f32 % LOOP_DURATION;
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.01, 0.01, 0.03, 1.0), depth: 1.0);
		let camera = self.camera(frame.viewport);
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
			self.add_text_plane(g, &mut buf, item.text, properties.scale, properties.plane_transform(), &properties.scribe());
		}
		g.end();
	}
}
