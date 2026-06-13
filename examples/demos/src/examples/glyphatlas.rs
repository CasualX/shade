use crate::*;
use std::fmt::Write;

const PANEL_WIDTH: f32 = 330.0;
const MARGIN: f32 = 24.0;
const MIN_ATLAS_SIZE: f32 = 80.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AtlasMode {
	Raw,
	Msdf,
}

impl AtlasMode {
	fn label(self) -> &'static str {
		match self {
			AtlasMode::Raw => "raw texture",
			AtlasMode::Msdf => "MTSDF shader",
		}
	}
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(GlyphAtlas::new(g, assets))
}

struct GlyphAtlas {
	font: d2::FontResource<shade::atlas::Font>,
	raw_texture: Box<dyn shade::Texture2D>,
	textured_shader: Box<dyn shade::ShaderProgram>,
	color_shader: Box<dyn shade::ShaderProgram>,
	viewport: Bounds2i,
	cursor: Vec2f,
	hovered: Option<u32>,
	selected: Option<u32>,
	atlas_mode: AtlasMode,
}

const FONT_DESC: &str = "font/font.json";
const FONT_TEXTURE: &str = "font/font.png";

impl GlyphAtlas {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> GlyphAtlas {
		let font = load_font(g, assets, FONT_DESC, FONT_TEXTURE, false);
		let raw_texture = {
			let data = assets.read(FONT_TEXTURE).expect("failed to load font texture");
			let image = shade::image::ImageRGBA::load_memory_png(&data).expect("failed to decode font texture");
			let image = image.map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
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
		GlyphAtlas {
			font,
			raw_texture,
			textured_shader,
			color_shader,
			viewport: Bounds2!(0, 0, 1, 1),
			cursor: Vec2::ZERO,
			hovered: None,
			selected: None,
			atlas_mode: AtlasMode::Raw,
		}
	}

	fn atlas_screen_rect(&self, viewport: Bounds2i) -> Bounds2f {
		let meta = &self.font.font.meta;
		let atlas_size = Vec2(meta.width as f32, meta.height as f32);
		let panel_width = if viewport.width() as f32 >= 720.0 { PANEL_WIDTH } else { 0.0 };
		let available = Vec2(
			(viewport.width() as f32 - panel_width - MARGIN * 3.0).max(MIN_ATLAS_SIZE),
			(viewport.height() as f32 - MARGIN * 2.0).max(MIN_ATLAS_SIZE),
		);
		let scale = (available.x / atlas_size.x).min(available.y / atlas_size.y).max(0.001);
		let size = atlas_size * scale;
		let left = MARGIN + (available.x - size.x) * 0.5;
		let top = MARGIN + (available.y - size.y) * 0.5;
		Bounds2!(left, top, left + size.x, top + size.y)
	}

	fn atlas_to_screen(&self, atlas: Vec2f, atlas_rect: Bounds2f) -> Vec2f {
		let meta = &self.font.font.meta;
		Vec2(
			atlas_rect.left() + atlas.x / meta.width as f32 * atlas_rect.width(),
			atlas_rect.top() + atlas.y / meta.height as f32 * atlas_rect.height(),
		)
	}

	fn screen_to_atlas(&self, position: Vec2f, atlas_rect: Bounds2f) -> Option<Vec2f> {
		if !atlas_rect.contains(position) {
			return None;
		}
		let meta = &self.font.font.meta;
		let local = position - atlas_rect.top_left();
		Some(Vec2(
			local.x / atlas_rect.width() * meta.width as f32,
			local.y / atlas_rect.height() * meta.height as f32,
		))
	}

	fn glyph_screen_rect(&self, glyph: &shade::atlas::Glyph, atlas_rect: Bounds2f) -> Option<Bounds2f> {
		let bounds = glyph.bounds?.atlas_bounds;
		let top_left = self.atlas_to_screen(Vec2(bounds.left() as f32, bounds.top() as f32), atlas_rect);
		let bottom_right = self.atlas_to_screen(Vec2(bounds.right() as f32, bounds.bottom() as f32), atlas_rect);
		Some(Bounds2!(top_left.x, top_left.y, bottom_right.x, bottom_right.y))
	}

	fn pick_glyph(&self, position: Vec2f) -> Option<u32> {
		let atlas_rect = self.atlas_screen_rect(self.viewport);
		let atlas_pos = self.screen_to_atlas(position, atlas_rect)?;
		let mut best: Option<(u32, f32)> = None;
		for (index, glyph) in self.font.font.glyphs.iter().enumerate() {
			let Some(bounds) = glyph.bounds.map(|bounds| bounds.atlas_bounds) else { continue };
			if atlas_pos.x < bounds.left() as f32 || atlas_pos.x > bounds.right() as f32 || atlas_pos.y < bounds.top() as f32 || atlas_pos.y > bounds.bottom() as f32 {
				continue;
			}
			let area = (bounds.width * bounds.height) as f32;
			match best {
				None => best = Some((index as u32, area)),
				Some((_, best_area)) if area < best_area => best = Some((index as u32, area)),
				_ => {}
			}
		}
		best.map(|(index, _)| index)
	}

	fn glyph_chars(&self, glyph_index: u32) -> Vec<char> {
		let mut chars = self.font.font.codepoints
			.iter()
			.filter_map(|(&chr, &index)| (index == glyph_index).then_some(chr))
			.collect::<Vec<_>>();
		chars.sort_unstable();
		chars
	}

	fn glyph_label(&self, glyph_index: u32) -> String {
		let chars = self.glyph_chars(glyph_index);
		if chars.is_empty() {
			return "unmapped".to_owned();
		}
		chars.into_iter()
			.map(|chr| {
				if chr.is_control() {
					format!("U+{:04X}", chr as u32)
				}
				else {
					format!("'{}' U+{:04X}", chr, chr as u32)
				}
			})
			.collect::<Vec<_>>()
			.join(", ")
	}

	fn selected_text(&self) -> String {
		let font = &self.font.font;
		let meta = &font.meta;
		let mut text = format!(
			"font atlas\nmode: {}\n{} x {} px\nkind: {:?}\nglyphs: {}\nkerning pairs: {}\nrange: {:.2}\n\n",
			self.atlas_mode.label(),
			meta.width,
			meta.height,
			meta.kind,
			font.glyphs.len(),
			font.kerning.len(),
			meta.distance_range,
		);
		let Some(index) = self.selected.or(self.hovered) else {
			text.push_str("click a glyph");
			return text;
		};
		let Some(glyph) = font.glyphs.get(index as usize) else {
			text.push_str("glyph is out of range");
			return text;
		};

		let _ = writeln!(text, "glyph #{}", index);
		let _ = writeln!(text, "char: {}", self.glyph_label(index));
		let _ = writeln!(text, "metrics index: {}", glyph.metrics_index);
		let _ = writeln!(text, "advance: {:.4}", glyph.advance);
		if let Some(bounds) = glyph.bounds.map(|bounds| bounds.atlas_bounds) {
			let _ = writeln!(
				text,
				"atlas bounds:\n  x {}  y {}\n  {} x {} px",
				bounds.x,
				bounds.y,
				bounds.width,
				bounds.height,
			);
		}
		else {
			text.push_str("atlas bounds: none\n");
		}
		if let Some(bounds) = glyph.bounds.map(|bounds| bounds.plane_bounds) {
			let _ = writeln!(
				text,
				"plane bounds:\n  l {:.4}  t {:.4}\n  r {:.4}  b {:.4}",
				bounds.left,
				bounds.top,
				bounds.right,
				bounds.bottom,
			);
		}
		else {
			text.push_str("plane bounds: none\n");
		}
		if let Some(metrics) = font.metrics.get(glyph.metrics_index as usize) {
			let _ = writeln!(
				text,
				"metrics:\n  em {:.3}\n  line {:.3}\n  asc {:.3}\n  desc {:.3}",
				metrics.em_size,
				metrics.line_height,
				metrics.ascender,
				metrics.descender,
			);
		}
		text
	}

	fn draw_atlas_raw(&self, g: &mut shade::Graphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
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

	fn draw_atlas_msdf(&self, g: &mut shade::Graphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		let meta = &self.font.font.meta;
		let mut buf = d2::TextBuffer::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.shader = Some(&*self.font.shader);
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		buf.uniform.texture = &*self.font.texture;
		buf.uniform.unit_range = Vec2::dup(meta.distance_range).cast::<f32>() / Vec2(meta.width, meta.height).cast::<f32>();
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

	fn draw_atlas(&self, g: &mut shade::Graphics, viewport: Bounds2i, atlas_rect: Bounds2f) {
		match self.atlas_mode {
			AtlasMode::Raw => self.draw_atlas_raw(g, viewport, atlas_rect),
			AtlasMode::Msdf => self.draw_atlas_msdf(g, viewport, atlas_rect),
		}
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

		let glyph_border = d2::Paint {
			template: d2::ColorTemplate {
				color1: Vec4(80, 180, 255, 95),
				color2: Vec4(80, 180, 255, 95),
			},
		};
		for glyph in &self.font.font.glyphs {
			if let Some(rect) = self.glyph_screen_rect(glyph, atlas_rect) {
				buf.fill_edge_rect(&glyph_border, &rect, 1.0);
			}
		}

		if let Some(index) = self.hovered {
			if let Some(rect) = self.font.font.glyphs.get(index as usize).and_then(|glyph| self.glyph_screen_rect(glyph, atlas_rect)) {
				let paint = d2::Paint {
					template: d2::ColorTemplate {
						color1: Vec4(255, 225, 80, 240),
						color2: Vec4(255, 225, 80, 240),
					},
				};
				buf.fill_edge_rect(&paint, &rect, 2.0);
			}
		}
		if let Some(index) = self.selected {
			if let Some(rect) = self.font.font.glyphs.get(index as usize).and_then(|glyph| self.glyph_screen_rect(glyph, atlas_rect)) {
				let paint = d2::Paint {
					template: d2::ColorTemplate {
						color1: Vec4(255, 80, 110, 255),
						color2: Vec4(255, 80, 110, 255),
					},
				};
				buf.fill_edge_rect(&paint, &rect, 2.0);
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

impl DemoInterface for GlyphAtlas {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn resize(&mut self, size: Vec2i) {
		self.viewport = Bounds2!(0, 0, size.x, size.y);
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position } => {
				self.cursor = position;
				self.hovered = self.pick_glyph(position);
				shell.set_cursor(if self.hovered.is_some() { Cursor::Pointer } else { Cursor::Default });
				shell.request_redraw();
			}
			Input::MouseButton { button: gui::MouseButton::LEFT, pressed: true, position } => {
				self.cursor = position;
				self.hovered = self.pick_glyph(position);
				self.selected = self.hovered;
				shell.request_redraw();
			}
			Input::KeyDown(Key::Digit1) => {
				self.atlas_mode = AtlasMode::Raw;
				shell.request_redraw();
			}
			Input::KeyDown(Key::Digit2) => {
				self.atlas_mode = AtlasMode::Msdf;
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
		self.draw_atlas(g, viewport, atlas_rect);
		self.draw_overlays(g, viewport, atlas_rect);
		self.draw_text(g, viewport);
		g.end();
	}
}
