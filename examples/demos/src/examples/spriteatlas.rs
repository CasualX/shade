use crate::*;
use std::fmt::Write;

const PANEL_WIDTH: f32 = 330.0;
const MARGIN: f32 = 24.0;
const MIN_ATLAS_SIZE: f32 = 80.0;
const MIN_ZOOM_LEVEL: i32 = -7;
const MAX_ZOOM_LEVEL: i32 = 7;
const ATLAS_DESC: &str = "atlas.json";
const ATLAS_TEXTURE: &str = "atlas.png";

#[derive(Clone, Debug)]
struct SpriteRegion {
	name: String,
	frame: usize,
	frame_count: usize,
	rect: shade::atlas::Rect,
	margin: i32,
	transform: shade::atlas::Transform,
	origin: Vec2i,
	duration: Option<f32>,
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(SpriteAtlas::new(g, assets))
}

struct SpriteAtlas {
	atlas: shade::atlas::Atlas,
	regions: Vec<SpriteRegion>,
	font: d2::FontResource<shade::atlas::Font>,
	texture: Box<dyn shade::Texture2D>,
	textured_shader: Box<dyn shade::ShaderProgram>,
	color_shader: Box<dyn shade::ShaderProgram>,
	viewport: Bounds2i,
	hovered: Option<usize>,
	selected: Option<usize>,
	zoom_level: i32,
	zoom_initialized: bool,
	pan: Vec2f,
	drag_cursor: Option<Vec2f>,
}

impl SpriteAtlas {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> SpriteAtlas {
		let atlas: shade::atlas::Atlas = {
			let text = assets.read_to_string(ATLAS_DESC).expect("failed to load atlas metadata");
			serde_json::from_str(&text).expect("failed to parse atlas metadata")
		};
		let texture = {
			let data = assets.read(ATLAS_TEXTURE).expect("failed to load atlas texture");
			let image = shade::image::ImageRGBA::load_memory_png(&data).expect("failed to decode atlas texture");
			assert_eq!(image.width, atlas.meta.width, "atlas texture width does not match metadata");
			assert_eq!(image.height, atlas.meta.height, "atlas texture height does not match metadata");
			let image = image.map_colors(|[r, g, b, a]| shade::color::Srgba8 { r, g, b, a });
			let props = shade::TextureProps! {
				usage: shade::TextureUsage::TEXTURE,
				filter: shade::TextureFilter::Nearest,
				wrap: shade::TextureWrap::Edge,
			};
			g.image(&props.bind(&image))
		};
		let mut shader_source = shade::shader_interface! {
			files {
				"textured.glsl" => shade::shaders::TEXTURED,
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let textured_shader = g.shader_compile(&mut shader_source, "textured.glsl", &[]);
		let color_shader = g.shader_compile(&mut shader_source, "color.glsl", &[]);
		let regions = Self::collect_regions(&atlas);
		let font = load_font(g, assets, "font/font.json", "font/font.png", false);

		SpriteAtlas {
			atlas,
			regions,
			font,
			texture,
			textured_shader,
			color_shader,
			viewport: Bounds2!(0, 0, 1, 1),
			hovered: None,
			selected: None,
			zoom_level: 0,
			zoom_initialized: false,
			pan: Vec2::ZERO,
			drag_cursor: None,
		}
	}

	fn collect_regions(atlas: &shade::atlas::Atlas) -> Vec<SpriteRegion> {
		let mut names = atlas.sprites.keys().cloned().collect::<Vec<_>>();
		names.sort();
		let mut regions = Vec::new();
		for name in names {
			match &atlas.sprites[&name] {
				shade::atlas::Sprite::Frame(frame) => {
					regions.push(SpriteRegion {
						name,
						frame: 0,
						frame_count: 1,
						rect: frame.rect,
						margin: frame.margin,
						transform: frame.transform,
						origin: frame.origin,
						duration: None,
					});
				}
				shade::atlas::Sprite::Animated(frames) => {
					for (index, animated) in frames.iter().enumerate() {
						regions.push(SpriteRegion {
							name: name.clone(),
							frame: index,
							frame_count: frames.len(),
							rect: animated.frame.rect,
							margin: animated.frame.margin,
							transform: animated.frame.transform,
							origin: animated.frame.origin,
							duration: Some(animated.duration),
						});
					}
				}
			}
		}
		regions
	}

	fn available_rect(viewport: Bounds2i) -> Bounds2f {
		let panel_width = if viewport.width() as f32 >= 720.0 { PANEL_WIDTH } else { 0.0 };
		Bounds2!(
			MARGIN,
			MARGIN,
			(viewport.right() as f32 - panel_width - MARGIN * 2.0).max(MARGIN + MIN_ATLAS_SIZE),
			(viewport.bottom() as f32 - MARGIN).max(MARGIN + MIN_ATLAS_SIZE),
		)
	}

	fn zoom_scale(&self) -> f32 {
		if self.zoom_level >= 0 {
			(self.zoom_level + 1) as f32
		}
		else {
			1.0 / (1 - self.zoom_level) as f32
		}
	}

	fn zoom_label(&self) -> String {
		if self.zoom_level >= 0 {
			format!("{}:1", self.zoom_level + 1)
		}
		else {
			format!("1:{}", 1 - self.zoom_level)
		}
	}

	fn fitted_zoom_level(&self, viewport: Bounds2i) -> i32 {
		let available = Self::available_rect(viewport);
		let atlas_size = Vec2(self.atlas.meta.width as f32, self.atlas.meta.height as f32);
		let fit = (available.width() / atlas_size.x).min(available.height() / atlas_size.y);
		if fit >= 1.0 {
			(fit.floor() as i32 - 1).clamp(0, MAX_ZOOM_LEVEL)
		}
		else {
			(1 - (1.0 / fit.max(0.001)).ceil() as i32).clamp(MIN_ZOOM_LEVEL, 0)
		}
	}

	fn atlas_screen_rect(&self, viewport: Bounds2i) -> Bounds2f {
		let available = Self::available_rect(viewport);
		let atlas_size = Vec2(self.atlas.meta.width as f32, self.atlas.meta.height as f32);
		let scale = self.zoom_scale();
		let size = atlas_size * scale;
		let left = (available.left() + (available.width() - size.x) * 0.5 + self.pan.x).round();
		let top = (available.top() + (available.height() - size.y) * 0.5 + self.pan.y).round();
		Bounds2!(left, top, left + size.x, top + size.y)
	}

	fn zoom_at(&mut self, position: Vec2f, direction: i32) {
		let old_rect = self.atlas_screen_rect(self.viewport);
		let old_scale = self.zoom_scale();
		let atlas_point = (position - old_rect.top_left()) / old_scale;
		let new_level = (self.zoom_level + direction).clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
		if new_level == self.zoom_level {
			return;
		}
		self.zoom_level = new_level;

		let available = Self::available_rect(self.viewport);
		let atlas_size = Vec2(self.atlas.meta.width as f32, self.atlas.meta.height as f32);
		let new_size = atlas_size * self.zoom_scale();
		let centered = Vec2(
			available.left() + (available.width() - new_size.x) * 0.5,
			available.top() + (available.height() - new_size.y) * 0.5,
		);
		self.pan = position - atlas_point * self.zoom_scale() - centered;
	}

	fn atlas_to_screen(&self, point: Vec2f, atlas_rect: Bounds2f) -> Vec2f {
		Vec2(
			atlas_rect.left() + point.x / self.atlas.meta.width as f32 * atlas_rect.width(),
			atlas_rect.top() + point.y / self.atlas.meta.height as f32 * atlas_rect.height(),
		)
	}

	fn screen_to_atlas(&self, point: Vec2f, atlas_rect: Bounds2f) -> Option<Vec2f> {
		if !atlas_rect.contains(point) {
			return None;
		}
		let local = point - atlas_rect.top_left();
		Some(Vec2(
			local.x / atlas_rect.width() * self.atlas.meta.width as f32,
			local.y / atlas_rect.height() * self.atlas.meta.height as f32,
		))
	}

	fn region_atlas_rect(&self, region: &SpriteRegion) -> shade::atlas::Rect {
		let margin = region.margin.max(0);
		let left = (region.rect.left() - margin).max(0);
		let top = (region.rect.top() - margin).max(0);
		let right = (region.rect.right() + margin).min(self.atlas.meta.width);
		let bottom = (region.rect.bottom() + margin).min(self.atlas.meta.height);
		shade::atlas::Rect(left, top, right - left, bottom - top)
	}

	fn region_screen_rect(&self, region: &SpriteRegion, atlas_rect: Bounds2f) -> Bounds2f {
		let rect = self.region_atlas_rect(region);
		let top_left = self.atlas_to_screen(Vec2(rect.left() as f32, rect.top() as f32), atlas_rect);
		let bottom_right = self.atlas_to_screen(Vec2(rect.right() as f32, rect.bottom() as f32), atlas_rect);
		Bounds2!(top_left.x, top_left.y, bottom_right.x, bottom_right.y)
	}

	fn pick_region(&self, position: Vec2f) -> Option<usize> {
		let atlas_rect = self.atlas_screen_rect(self.viewport);
		let point = self.screen_to_atlas(position, atlas_rect)?;
		self.regions
			.iter()
			.enumerate()
			.filter(|(_, region)| {
				let rect = self.region_atlas_rect(region);
				point.x >= rect.left() as f32
					&& point.x <= rect.right() as f32
					&& point.y >= rect.top() as f32
					&& point.y <= rect.bottom() as f32
			})
			.min_by_key(|(_, region)| {
				let rect = self.region_atlas_rect(region);
				rect.width * rect.height
			})
			.map(|(index, _)| index)
	}

	fn selected_text(&self) -> String {
		let animated = self.atlas.sprites.values().filter(|sprite| sprite.as_animated().is_some()).count();
		let mut text = format!(
			"sprite atlas\n{} x {} px\nzoom: {}\nkind: {:?}\nsprites: {}\nanimations: {}\nframes: {}\n\nwheel = zoom\nright drag = pan\n\n",
			self.atlas.meta.width,
			self.atlas.meta.height,
			self.zoom_label(),
			self.atlas.meta.kind,
			self.atlas.sprites.len(),
			animated,
			self.regions.len(),
		);
		let Some(index) = self.selected else {
			text.push_str("click a sprite frame");
			return text;
		};
		let Some(region) = self.regions.get(index) else {
			text.push_str("frame is out of range");
			return text;
		};
		let _ = writeln!(text, "sprite: {}", region.name);
		if region.frame_count > 1 {
			let _ = writeln!(text, "frame: {} / {}", region.frame + 1, region.frame_count);
		}
		else {
			text.push_str("frame: static\n");
		}
		let _ = writeln!(
			text,
			"atlas bounds:\n  x {}  y {}\n  {} x {} px",
			region.rect.x,
			region.rect.y,
			region.rect.width,
			region.rect.height,
		);
		let _ = writeln!(text, "margin: {} px", region.margin);
		let _ = writeln!(text, "origin: {}, {}", region.origin.x, region.origin.y);
		let _ = writeln!(text, "transform: {:?}", region.transform);
		if let Some(duration) = region.duration {
			let _ = writeln!(text, "duration: {:.3} s", duration);
		}
		text
	}

	fn draw_texture(&self, g: &mut shade::Graphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let mut buf = d2::TexturedBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.textured_shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		buf.uniform.texture = &*self.texture;
		let color = Vec4(255, 255, 255, 255);
		let sprite = d2::Sprite {
			bottom_left: d2::TexturedTemplate { uv: Vec2(0.0, 1.0), color },
			top_left: d2::TexturedTemplate { uv: Vec2(0.0, 0.0), color },
			top_right: d2::TexturedTemplate { uv: Vec2(1.0, 0.0), color },
			bottom_right: d2::TexturedTemplate { uv: Vec2(1.0, 1.0), color },
		};
		buf.sprite_rect(&sprite, &atlas_rect);
		buf.draw(g);
	}

	fn draw_overlays(&self, g: &mut shade::Graphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let mut buf = d2::ColorBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.color_shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());

		let panel_left = if viewport.width() as f32 >= 720.0 {
			viewport.right() as f32 - PANEL_WIDTH - MARGIN
		}
		else {
			MARGIN
		};
		let panel = Bounds2!(panel_left, MARGIN, viewport.right() as f32 - MARGIN, (viewport.bottom() as f32 - MARGIN).max(MARGIN + 180.0));
		let panel_paint = d2::Paint {
			template: d2::ColorTemplate {
				color1: Vec4(16, 18, 24, 210),
				color2: Vec4(16, 18, 24, 210),
			},
		};
		buf.fill_rect(&panel_paint, &panel);

		let atlas_border = d2::Paint {
			template: d2::ColorTemplate {
				color1: Vec4(255, 255, 255, 180),
				color2: Vec4(255, 255, 255, 180),
			},
		};
		buf.fill_edge_rect(&atlas_border, &atlas_rect, 1.0);

		let frame_border = d2::Paint {
			template: d2::ColorTemplate {
				color1: Vec4(80, 200, 255, 110),
				color2: Vec4(80, 200, 255, 110),
			},
		};
		for region in &self.regions {
			buf.fill_edge_rect(&frame_border, &self.region_screen_rect(region, atlas_rect), 1.0);
		}

		if let Some(index) = self.hovered {
			if let Some(region) = self.regions.get(index) {
				let paint = d2::Paint {
					template: d2::ColorTemplate {
						color1: Vec4(255, 225, 80, 240),
						color2: Vec4(255, 225, 80, 240),
					},
				};
				buf.fill_edge_rect(&paint, &self.region_screen_rect(region, atlas_rect), 2.0);
			}
		}
		if let Some(index) = self.selected {
			if let Some(region) = self.regions.get(index) {
				let paint = d2::Paint {
					template: d2::ColorTemplate {
						color1: Vec4(255, 80, 110, 255),
						color2: Vec4(255, 80, 110, 255),
					},
				};
				buf.fill_edge_rect(&paint, &self.region_screen_rect(region, atlas_rect), 2.0);
			}
		}
		buf.draw(g);
	}

	fn draw_text(&self, g: &mut shade::Graphics, viewport: Bounds2i) {
		let mut text = d2::TextBuffer::new();
		text.blend_mode = shade::BlendMode::Alpha;
		text.uniform.transform = Transform2::ortho(viewport.cast());
		text.uniform.outline_width_relative = 0.18;
		let panel_left = if viewport.width() as f32 >= 720.0 {
			viewport.right() as f32 - PANEL_WIDTH - MARGIN
		}
		else {
			MARGIN
		};
		let mut cursor = d2::Cursor(Vec2(panel_left + 18.0, MARGIN + 20.0));
		let mut scribe = d2::Scribe {
			font_size: 24.0,
			line_height: 24.0,
			x_pos: cursor.pos.x,
			color: Vec4(242, 244, 248, 255),
			..Default::default()
		};
		scribe.set_baseline_relative(0.0);
		text.text_write(&self.font, &mut scribe, &mut cursor, &self.selected_text());
		text.draw(g);
	}
}

impl DemoInterface for SpriteAtlas {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn resize(&mut self, size: Vec2i) {
		self.viewport = Bounds2!(0, 0, size.x, size.y);
		if !self.zoom_initialized {
			self.zoom_level = self.fitted_zoom_level(self.viewport);
			self.zoom_initialized = true;
		}
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position } => {
				if let Some(previous) = self.drag_cursor {
					self.drag_cursor = Some(position);
					self.pan += position - previous;
					self.hovered = None;
					shell.set_cursor(Cursor::Grabbing);
				}
				else {
					self.hovered = self.pick_region(position);
					if let Some(index) = self.hovered {
						self.selected = Some(index);
					}
					shell.set_cursor(if self.hovered.is_some() { Cursor::Pointer } else { Cursor::Default });
				}
				shell.request_redraw();
			}
			Input::MouseButton { button: gui::MouseButton::LEFT, pressed: true, position } => {
				self.hovered = self.pick_region(position);
				if let Some(index) = self.hovered {
					self.selected = Some(index);
				}
				else {
					self.selected = None;
				}
				shell.request_redraw();
			}
			Input::MouseButton { button: gui::MouseButton::RIGHT, pressed, position } => {
				if pressed {
					self.drag_cursor = Some(position);
					shell.set_cursor(Cursor::Grabbing);
				}
				else {
					self.drag_cursor = None;
					self.hovered = self.pick_region(position);
					shell.set_cursor(if self.hovered.is_some() { Cursor::Pointer } else { Cursor::Grab });
				}
				shell.request_redraw();
			}
			Input::MouseWheel { delta, position } => {
				if delta.y != 0.0 {
					self.zoom_at(position, if delta.y > 0.0 { 1 } else { -1 });
					self.hovered = self.pick_region(position);
					if let Some(index) = self.hovered {
						self.selected = Some(index);
					}
				}
				shell.request_redraw();
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		self.viewport = viewport;
		let atlas_rect = self.atlas_screen_rect(viewport);

		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.075, 0.08, 0.095, 1.0));
		self.draw_texture(g, viewport, atlas_rect);
		self.draw_overlays(g, viewport, atlas_rect);
		self.draw_text(g, viewport);
		g.end();
	}
}
