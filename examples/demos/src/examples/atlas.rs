use crate::*;
use std::fmt::Write;

const ATLAS_DESC: &str = "font/font.json";
const ATLAS_TEXTURE: &str = "font/font.png";
const HUD_FONT: &str = "font";

const PANEL_WIDTH: f32 = 330.0;
const MARGIN: f32 = 24.0;
const MIN_ATLAS_SIZE: f32 = 80.0;
const MIN_ZOOM_LEVEL: i32 = -7;
const MAX_ZOOM_LEVEL: i32 = 7;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TextureView {
	Raw,
	DistanceField,
}

#[derive(Clone, Debug)]
struct SpriteRegion {
	name: String,
	frame: usize,
	frame_count: usize,
	rect: Recti,
	margin: i32,
	transform: shade::atlas::Transform,
	origin: Vec2i,
	duration: Option<f32>,
}

#[derive(Clone, Debug)]
struct GlyphRegion {
	font: String,
	index: u32,
}

#[derive(Clone, Debug)]
enum RegionKind {
	Sprite(SpriteRegion),
	Glyph(GlyphRegion),
}

#[derive(Clone, Debug)]
struct Region {
	kind: RegionKind,
	/// Atlas area used for picking and drawing the region overlay.
	bounds: Recti,
}

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(AtlasViewer::new(g, assets))
}

struct AtlasViewer {
	atlas: shade::atlas::Atlas,
	regions: Vec<Region>,
	hud_font: d2::FontResource<shade::atlas::Font>,
	raw_texture: Box<dyn shade::Texture2D>,
	distance_texture: Box<dyn shade::Texture2D>,
	textured_shader: Box<dyn shade::ShaderProgram>,
	distance_shader: Box<dyn shade::ShaderProgram>,
	color_shader: Box<dyn shade::ShaderProgram>,
	viewport: Bounds2i,
	cursor: Vec2f,
	hovered: Option<usize>,
	selected: Option<usize>,
	texture_view: TextureView,
	zoom_level: i32,
	zoom_initialized: bool,
	pan: Vec2f,
	drag_cursor: Option<Vec2f>,
}

impl AtlasViewer {
	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> AtlasViewer {
		let atlas: shade::atlas::Atlas = {
			let text = assets.read_to_string(ATLAS_DESC).expect("failed to load atlas metadata");
			serde_json::from_str(&text).expect("failed to parse atlas metadata")
		};
		let (raw_texture, distance_texture) = Self::load_textures(g, assets, &atlas);
		let mut shader_source = shade::shader_interface! {
			files {
				"textured.glsl" => shade::shaders::TEXTURED,
				"mtsdf.glsl" => shade::shaders::MTSDF,
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let textured_shader = g.shader_compile(&mut shader_source, "textured.glsl", &[]);
		let distance_shader = g.shader_compile(&mut shader_source, "mtsdf.glsl", &[]);
		let color_shader = g.shader_compile(&mut shader_source, "color.glsl", &[]);
		let regions = Self::collect_regions(&atlas);
		let hud_font = load_atlas_font(g, assets, ATLAS_DESC, ATLAS_TEXTURE, HUD_FONT, false).1;
		let texture_view = if atlas.meta.kind == shade::atlas::Kind::Bitmap {
			TextureView::Raw
		}
		else {
			TextureView::DistanceField
		};

		AtlasViewer {
			atlas,
			regions,
			hud_font,
			raw_texture,
			distance_texture,
			textured_shader,
			distance_shader,
			color_shader,
			viewport: Bounds2!(0, 0, 1, 1),
			cursor: Vec2::ZERO,
			hovered: None,
			selected: None,
			texture_view,
			zoom_level: 0,
			zoom_initialized: false,
			pan: Vec2::ZERO,
			drag_cursor: None,
		}
	}

	fn load_textures(
		g: &mut dyn shade::IGraphics,
		assets: &dyn AssetLoader,
		atlas: &shade::atlas::Atlas,
	) -> (Box<dyn shade::Texture2D>, Box<dyn shade::Texture2D>) {
		let data = assets.read(ATLAS_TEXTURE).expect("failed to load atlas texture");
		let image = shade::image::ImageRGBA::load_memory_png(&data).expect("failed to decode atlas texture");
		assert_eq!(image.width, atlas.meta.width, "atlas texture width does not match metadata");
		assert_eq!(image.height, atlas.meta.height, "atlas texture height does not match metadata");

		let raw_props = shade::TextureProps! {
			usage: shade::TextureUsage::TEXTURE,
			filter: shade::TextureFilter::Nearest,
			wrap: shade::TextureWrap::Edge,
		};
		let raw_texture = if atlas.meta.kind == shade::atlas::Kind::Bitmap {
			let image = image.clone().map_colors(|[r, g, b, a]| shade::color::Srgba8 { r, g, b, a });
			g.image(&raw_props.bind(&image))
		}
		else {
			let image = image.clone().map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
			g.image(&raw_props.bind(&image))
		};

		let distance_props = shade::TextureProps! {
			usage: shade::TextureUsage::TEXTURE,
			filter: shade::TextureFilter::Linear,
			wrap: shade::TextureWrap::Edge,
		};
		let image = image.map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
		let distance_texture = g.image(&distance_props.bind(&image));
		(raw_texture, distance_texture)
	}

	fn collect_regions(atlas: &shade::atlas::Atlas) -> Vec<Region> {
		let mut regions = Vec::new();
		let mut sprite_names = atlas.sprites.keys().cloned().collect::<Vec<_>>();
		sprite_names.sort();
		for name in sprite_names {
			match &atlas.sprites[&name] {
				shade::atlas::Sprite::Frame(frame) => {
					regions.push(Self::sprite_region(name, 0, 1, frame, None, &atlas.meta));
				}
				shade::atlas::Sprite::Animated(frames) => {
					for (index, animated) in frames.iter().enumerate() {
						regions.push(Self::sprite_region(
							name.clone(),
							index,
							frames.len(),
							&animated.frame,
							Some(animated.duration),
							&atlas.meta,
						));
					}
				}
			}
		}

		let mut font_names = atlas.fonts.keys().cloned().collect::<Vec<_>>();
		font_names.sort();
		for font_name in font_names {
			for (index, glyph) in atlas.fonts[&font_name].glyphs.iter().enumerate() {
				let Some(bounds) = glyph.bounds.map(|bounds| bounds.atlas_bounds) else { continue };
				regions.push(Region {
					kind: RegionKind::Glyph(GlyphRegion {
						font: font_name.clone(),
						index: index as u32,
					}),
					bounds,
				});
			}
		}
		regions
	}

	fn sprite_region(
		name: String,
		frame_index: usize,
		frame_count: usize,
		frame: &shade::atlas::Frame,
		duration: Option<f32>,
		meta: &shade::atlas::Metadata,
	) -> Region {
		let margin = frame.margin.max(0);
		let left = (frame.rect.left() - margin).max(0);
		let top = (frame.rect.top() - margin).max(0);
		let right = (frame.rect.right() + margin).min(meta.width);
		let bottom = (frame.rect.bottom() + margin).min(meta.height);
		Region {
			kind: RegionKind::Sprite(SpriteRegion {
				name,
				frame: frame_index,
				frame_count,
				rect: frame.rect,
				margin: frame.margin,
				transform: frame.transform,
				origin: frame.origin,
				duration,
			}),
			bounds: Recti(left, top, right - left, bottom - top),
		}
	}

	fn supports_distance_view(&self) -> bool {
		self.atlas.meta.kind != shade::atlas::Kind::Bitmap && self.atlas.meta.distance_range > 0.0
	}

	fn texture_view_label(&self) -> &'static str {
		match self.texture_view {
			TextureView::Raw => "raw texture",
			TextureView::DistanceField => "decoded distance field",
		}
	}

	fn available_rect(viewport: Bounds2i) -> Bounds2f {
		let panel_width = if viewport.width() as f32 >= 720.0 { PANEL_WIDTH + MARGIN } else { 0.0 };
		Bounds2!(
			MARGIN,
			MARGIN,
			(viewport.right() as f32 - panel_width - MARGIN).max(MARGIN + MIN_ATLAS_SIZE),
			(viewport.bottom() as f32 - MARGIN).max(MARGIN + MIN_ATLAS_SIZE),
		)
	}

	fn panel_rect(viewport: Bounds2i) -> Bounds2f {
		let left = if viewport.width() as f32 >= 720.0 {
			viewport.right() as f32 - PANEL_WIDTH - MARGIN
		}
		else {
			MARGIN
		};
		Bounds2!(
			left,
			MARGIN,
			(viewport.right() as f32 - MARGIN).max(left + 180.0),
			(viewport.bottom() as f32 - MARGIN).max(MARGIN + 180.0),
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

	fn fit_view(&mut self) {
		self.zoom_level = self.fitted_zoom_level(self.viewport);
		self.pan = Vec2::ZERO;
	}

	fn atlas_screen_rect(&self, viewport: Bounds2i) -> Bounds2f {
		let available = Self::available_rect(viewport);
		let atlas_size = Vec2(self.atlas.meta.width as f32, self.atlas.meta.height as f32);
		let size = atlas_size * self.zoom_scale();
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

	fn region_screen_rect(&self, region: &Region, atlas_rect: Bounds2f) -> Bounds2f {
		let top_left = self.atlas_to_screen(Vec2(region.bounds.left() as f32, region.bounds.top() as f32), atlas_rect);
		let bottom_right = self.atlas_to_screen(Vec2(region.bounds.right() as f32, region.bounds.bottom() as f32), atlas_rect);
		Bounds2!(top_left.x, top_left.y, bottom_right.x, bottom_right.y)
	}

	fn pick_region(&self, position: Vec2f) -> Option<usize> {
		let atlas_rect = self.atlas_screen_rect(self.viewport);
		let point = self.screen_to_atlas(position, atlas_rect)?;
		self.regions
			.iter()
			.enumerate()
			.filter(|(_, region)| {
				point.x >= region.bounds.left() as f32
					&& point.x <= region.bounds.right() as f32
					&& point.y >= region.bounds.top() as f32
					&& point.y <= region.bounds.bottom() as f32
			})
			.min_by_key(|(_, region)| region.bounds.width.max(0) * region.bounds.height.max(0))
			.map(|(index, _)| index)
	}

	fn select_relative(&mut self, delta: isize) {
		if self.regions.is_empty() {
			self.selected = None;
			return;
		}
		let next = match self.selected {
			Some(current) => (current as isize + delta).rem_euclid(self.regions.len() as isize) as usize,
			None if delta < 0 => self.regions.len() - 1,
			None => 0,
		};
		self.selected = Some(next);
		self.hovered = None;
	}

	fn glyph_chars(&self, font_name: &str, glyph_index: u32) -> Vec<char> {
		let Some(font) = self.atlas.fonts.get(font_name) else { return Vec::new() };
		let mut chars = font.codepoints
			.iter()
			.filter_map(|(&chr, &index)| (index == glyph_index).then_some(chr))
			.collect::<Vec<_>>();
		chars.sort_unstable();
		chars
	}

	fn glyph_label(&self, font_name: &str, glyph_index: u32) -> String {
		let chars = self.glyph_chars(font_name, glyph_index);
		if chars.is_empty() {
			return "unmapped".to_owned();
		}
		chars.into_iter()
			.map(|chr| {
				if chr.is_control() {
					format!("U+{:04X}", chr as u32)
				}
				else {
					format!("'{}' U+{:04X}", chr.escape_default(), chr as u32)
				}
			})
			.collect::<Vec<_>>()
			.join(", ")
	}

	fn selected_text(&self) -> String {
		let animation_count = self.atlas.sprites.values().filter(|sprite| sprite.as_animated().is_some()).count();
		let frame_count = self.atlas.sprites.values().map(shade::atlas::Sprite::len).sum::<usize>();
		let glyph_count = self.atlas.fonts.values().map(|font| font.glyphs.len()).sum::<usize>();
		let mut text = format!(
			"atlas viewer\n{} x {} px  {}\nview: {}\nzoom: {}\nversion: {}\nsprites: {}  animations: {}\nframes: {}  fonts: {}  glyphs: {}\n\n1 raw  2 decoded\nP fit  wheel / up/down zoom\nright drag pan  left/right browse\n\n",
			self.atlas.meta.width,
			self.atlas.meta.height,
			self.atlas.meta.kind,
			self.texture_view_label(),
			self.zoom_label(),
			self.atlas.version,
			self.atlas.sprites.len(),
			animation_count,
			frame_count,
			self.atlas.fonts.len(),
			glyph_count,
		);
		let Some(index) = self.selected.or(self.hovered) else {
			text.push_str("hover or click a region");
			return text;
		};
		let Some(region) = self.regions.get(index) else {
			text.push_str("selection is out of range");
			return text;
		};

		match &region.kind {
			RegionKind::Sprite(sprite) => {
				let _ = writeln!(text, "sprite: {}", sprite.name);
				if sprite.frame_count > 1 {
					let _ = writeln!(text, "frame: {} / {}", sprite.frame + 1, sprite.frame_count);
				}
				else {
					text.push_str("frame: static\n");
				}
				let _ = writeln!(
					text,
					"bounds: x {}  y {}  {} x {} px",
					sprite.rect.x,
					sprite.rect.y,
					sprite.rect.width,
					sprite.rect.height,
				);
				let _ = writeln!(text, "margin: {} px", sprite.margin);
				let _ = writeln!(text, "origin: {}, {}", sprite.origin.x, sprite.origin.y);
				let _ = writeln!(text, "transform: {:?}", sprite.transform);
				if let Some(duration) = sprite.duration {
					let _ = writeln!(text, "duration: {:.3} s", duration);
				}
			}
			RegionKind::Glyph(glyph_region) => {
				let Some(font) = self.atlas.fonts.get(&glyph_region.font) else {
					text.push_str("font is missing");
					return text;
				};
				let Some(glyph) = font.glyphs.get(glyph_region.index as usize) else {
					text.push_str("glyph is out of range");
					return text;
				};
				let _ = writeln!(text, "font: {}", glyph_region.font);
				let _ = writeln!(text, "glyph #{}", glyph_region.index);
				let _ = writeln!(text, "char: {}", self.glyph_label(&glyph_region.font, glyph_region.index));
				let _ = writeln!(text, "metrics index: {}", glyph.metrics_index);
				let _ = writeln!(text, "advance: {:.4}", glyph.advance);
				if let Some(bounds) = glyph.bounds {
					let atlas = bounds.atlas_bounds;
					let plane = bounds.plane_bounds;
					let _ = writeln!(text, "bounds: x {}  y {}  {} x {} px", atlas.x, atlas.y, atlas.width, atlas.height);
					let _ = writeln!(text, "plane: {:.3}, {:.3} to {:.3}, {:.3}", plane.left, plane.top, plane.right, plane.bottom);
				}
				if let Some(metrics) = font.metrics.get(glyph.metrics_index as usize) {
					let _ = writeln!(
						text,
						"metrics: em {:.3}  line {:.3}\nasc {:.3}  desc {:.3}",
						metrics.em_size,
						metrics.line_height,
						metrics.ascender,
						metrics.descender,
					);
				}
			}
		}
		text
	}

	fn draw_raw_texture(&self, g: &mut dyn shade::IGraphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let mut buf = d2::TexturedBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.textured_shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		buf.uniform.texture = &*self.raw_texture;
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

	fn draw_distance_field(&self, g: &mut dyn shade::IGraphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let meta = &self.atlas.meta;
		let mut buf = d2::TextBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.distance_shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		buf.uniform.texture = &*self.distance_texture;
		buf.uniform.unit_range = Vec2::dup(meta.distance_range) / Vec2(meta.width as f32, meta.height as f32);
		buf.uniform.threshold = 0.5 - meta.distance_range_middle / meta.distance_range;
		buf.uniform.outline_width_absolute = 0.0;
		buf.uniform.outline_width_relative = 0.0;

		let color = Vec4(255, 255, 255, 255);
		let outline = Vec4(0, 0, 0, 0);
		let vertices = [
			d2::TextVertex {
				pos: atlas_rect.bottom_left(),
				uv: Vec2(0.0, 1.0),
				color,
				outline,
				data: d2::TextVertexData::BOTTOM_LEFT,
			},
			d2::TextVertex {
				pos: atlas_rect.top_left(),
				uv: Vec2(0.0, 0.0),
				color,
				outline,
				data: d2::TextVertexData::TOP_LEFT,
			},
			d2::TextVertex {
				pos: atlas_rect.top_right(),
				uv: Vec2(1.0, 0.0),
				color,
				outline,
				data: d2::TextVertexData::TOP_RIGHT,
			},
			d2::TextVertex {
				pos: atlas_rect.bottom_right(),
				uv: Vec2(1.0, 1.0),
				color,
				outline,
				data: d2::TextVertexData::BOTTOM_RIGHT,
			},
		];
		{
			let mut prim = buf.begin(shade::PrimType::Triangles, 4, 2);
			prim.add_indices_quad();
			prim.add_vertices(&vertices);
		}
		buf.draw(g);
	}

	fn draw_texture(&self, g: &mut dyn shade::IGraphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		match self.texture_view {
			TextureView::Raw => self.draw_raw_texture(g, viewport, atlas_rect),
			TextureView::DistanceField => self.draw_distance_field(g, viewport, atlas_rect),
		}
	}

	fn draw_overlays(&self, g: &mut dyn shade::IGraphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let mut buf = d2::ColorBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.color_shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());

		let atlas_border = d2::Paint {
			template: d2::ColorTemplate {
				color1: Vec4(255, 255, 255, 180),
				color2: Vec4(255, 255, 255, 180),
			},
		};
		buf.fill_edge_rect(&atlas_border, &atlas_rect, 1.0);

		for region in &self.regions {
			let color = match region.kind {
				RegionKind::Sprite(_) => Vec4(70, 190, 255, 105),
				RegionKind::Glyph(_) => Vec4(95, 235, 155, 95),
			};
			let paint = d2::Paint {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			buf.fill_edge_rect(&paint, &self.region_screen_rect(region, atlas_rect), 1.0);
		}

		if let Some(index) = self.hovered
			&& let Some(region) = self.regions.get(index)
		{
			let color = Vec4(255, 225, 80, 240);
			let paint = d2::Paint {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			buf.fill_edge_rect(&paint, &self.region_screen_rect(region, atlas_rect), 2.0);
		}
		if let Some(index) = self.selected
			&& let Some(region) = self.regions.get(index)
		{
			let color = Vec4(255, 80, 120, 255);
			let paint = d2::Paint {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			buf.fill_edge_rect(&paint, &self.region_screen_rect(region, atlas_rect), 2.0);
		}

		let panel = Self::panel_rect(viewport);
		let panel_color = Vec4(16, 18, 24, 220);
		let panel_paint = d2::Paint {
			template: d2::ColorTemplate { color1: panel_color, color2: panel_color },
		};
		buf.fill_rect(&panel_paint, &panel);
		buf.draw(g);
	}

	fn draw_text(&self, g: &mut dyn shade::IGraphics, viewport: Bounds2i) {
		let mut text = d2::TextBuffer::new();
		text.blend_mode = shade::BlendMode::Alpha;
		text.uniform.transform = Transform2::ortho(viewport.cast());
		text.uniform.outline_width_relative = 0.18;
		let panel = Self::panel_rect(viewport);
		let mut cursor = d2::Cursor(panel.top_left() + Vec2(18.0, 20.0));
		let mut scribe = d2::Scribe {
			font_size: 20.0,
			line_height: 21.0,
			x_pos: cursor.pos.x,
			color: Vec4(242, 244, 248, 255),
			..Default::default()
		};
		scribe.set_baseline_relative(0.0);
		text.text_write(&self.hud_font, &mut scribe, &mut cursor, self.selected_text());
		text.draw(g);
	}

	fn update_hover_and_cursor(&mut self, shell: &mut dyn ShellServices) {
		self.hovered = self.pick_region(self.cursor);
		let cursor = if self.hovered.is_some() {
			Cursor::Pointer
		}
		else if self.atlas_screen_rect(self.viewport).contains(self.cursor) {
			Cursor::Grab
		}
		else {
			Cursor::Default
		};
		shell.set_cursor(cursor);
	}
}

impl DemoInterface for AtlasViewer {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn resize(&mut self, size: Vec2i) {
		self.viewport = Bounds2!(0, 0, size.x, size.y);
		if !self.zoom_initialized {
			self.fit_view();
			self.zoom_initialized = true;
		}
	}

	fn input(&mut self, input: Input, _g: &mut dyn shade::IGraphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position } => {
				self.cursor = position;
				if let Some(previous) = self.drag_cursor {
					self.drag_cursor = Some(position);
					self.pan += position - previous;
					self.hovered = None;
					shell.set_cursor(Cursor::Grabbing);
				}
				else {
					self.update_hover_and_cursor(shell);
				}
			}
			Input::MouseButton { button, pressed, position } => {
				if button == gui::MouseButton::LEFT && pressed {
					self.cursor = position;
					self.hovered = self.pick_region(position);
					self.selected = self.hovered;
					self.update_hover_and_cursor(shell);
				}
				if button == gui::MouseButton::RIGHT || button == gui::MouseButton::MIDDLE {
					self.cursor = position;
					if pressed {
						self.drag_cursor = Some(position);
						self.hovered = None;
						shell.set_cursor(Cursor::Grabbing);
					}
					else {
						self.drag_cursor = None;
						self.update_hover_and_cursor(shell);
					}
				}
			}
			Input::MouseWheel { delta, position } => {
				self.cursor = position;
				if delta.y != 0.0 {
					self.zoom_at(position, if delta.y > 0.0 { 1 } else { -1 });
				}
				self.update_hover_and_cursor(shell);
			}
			Input::KeyDown(Key::Digit1) => {
				self.texture_view = TextureView::Raw;
			}
			Input::KeyDown(Key::Digit2) => {
				if self.supports_distance_view() {
					self.texture_view = TextureView::DistanceField;
				}
			}
			Input::KeyDown(Key::P) => {
				self.fit_view();
				self.update_hover_and_cursor(shell);
			}
			Input::KeyDown(Key::ArrowLeft) => self.select_relative(-1),
			Input::KeyDown(Key::ArrowRight) => self.select_relative(1),
			Input::KeyDown(Key::ArrowUp) => {
				let center = Self::available_rect(self.viewport).center();
				self.zoom_at(center, 1);
				self.update_hover_and_cursor(shell);
			}
			Input::KeyDown(Key::ArrowDown) => {
				let center = Self::available_rect(self.viewport).center();
				self.zoom_at(center, -1);
				self.update_hover_and_cursor(shell);
			}
			_ => return,
		}
		shell.request_redraw();
	}

	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
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
