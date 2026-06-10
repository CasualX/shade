use crate::*;

const INFLATE_PX: i32 = 5;

#[derive(Copy, Clone, Eq, PartialEq)]
enum DragEdge {
	Move,
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,
	Left,
	Right,
	Top,
	Bottom,
}

impl DragEdge {
	fn cursor(self) -> Cursor {
		match self {
			DragEdge::Move => Cursor::Move,
			DragEdge::TopLeft | DragEdge::BottomRight => Cursor::ResizeNwse,
			DragEdge::TopRight | DragEdge::BottomLeft => Cursor::ResizeNesw,
			DragEdge::Left | DragEdge::Right => Cursor::ResizeEastWest,
			DragEdge::Top | DragEdge::Bottom => Cursor::ResizeNorthSouth,
		}
	}
}

#[derive(Copy, Clone, PartialEq)]
struct PanelInstance {
	texture: shade::Texture2D,
	shader: shade::ShaderProgram,
	uv_x: &'static [f32],
	uv_y: &'static [f32],
	template_x: &'static [d2::layout::Unit],
	template_y: &'static [d2::layout::Unit],
	bounds: Bounds2i,
	min_size: Vec2i,
}

impl PanelInstance {
	fn hit_test(&self, pos: Vec2i) -> Option<DragEdge> {
		if Bounds2::point(self.bounds.top_left(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::TopLeft);
		}
		if Bounds2::point(self.bounds.top_right(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::TopRight);
		}
		if Bounds2::point(self.bounds.bottom_left(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::BottomLeft);
		}
		if Bounds2::point(self.bounds.bottom_right(), Vec2!(INFLATE_PX)).contains(pos) {
			return Some(DragEdge::BottomRight);
		}
		fn vertical(v: i32, b: Bounds2i) -> Bounds2i {
			Bounds2(Point2(v - INFLATE_PX, b.top()), Point2(v + INFLATE_PX, b.bottom()))
		}
		fn horizontal(v: i32, b: Bounds2i) -> Bounds2i {
			Bounds2(Point2(b.left(), v - INFLATE_PX), Point2(b.right(), v + INFLATE_PX))
		}
		if vertical(self.bounds.left(), self.bounds).contains(pos) {
			return Some(DragEdge::Left);
		}
		if vertical(self.bounds.right(), self.bounds).contains(pos) {
			return Some(DragEdge::Right);
		}
		if horizontal(self.bounds.top(), self.bounds).contains(pos) {
			return Some(DragEdge::Top);
		}
		if horizontal(self.bounds.bottom(), self.bounds).contains(pos) {
			return Some(DragEdge::Bottom);
		}
		if self.bounds.contains(pos) {
			return Some(DragEdge::Move);
		}
		None
	}
}

#[derive(Copy, Clone, PartialEq)]
struct DragState {
	edge: DragEdge,
	index: usize,
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Panels::new(g, assets))
}

struct Panels {
	panels: Vec<PanelInstance>,
	drag_state: Option<DragState>,
	cursor: Vec2i,
	line_shader: shade::ShaderProgram,
}

fn flex_panel(min: f32, max: f32, template: &[d2::layout::Unit]) -> Vec<f32> {
	let mut spans = vec![[0.0; 2]; template.len()];
	d2::layout::flex1d_slice(min, max, None, d2::layout::Justify::Start, template, &mut spans);
	let mut pos = Vec::with_capacity(spans.len() + 1);
	pos.push(min);
	for span in spans {
		pos.push(span[1]);
	}
	pos
}

impl Panels {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Panels {
		let mut panels = Vec::new();
		let mut shader_interface = shade::shader_interface! {
			files {
				"textured.glsl" => shade::shaders::TEXTURED,
				"pixelart.glsl" => shade::shaders::PIXELART,
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let texture_props = shade::TextureProps {
			mip_levels: 1,
			usage: shade::TextureUsage::TEXTURE,
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			wrap_u: shade::TextureWrap::Edge,
			wrap_v: shade::TextureWrap::Edge,
			..Default::default()
		};

		{
			use d2::layout::Unit::*;
			let texture = {
				let bytes = assets.read("textures/panel.png").unwrap();
				let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
				g.image(&texture_props.bind(&image))
			};
			let shader = g.shader_compile(&mut shader_interface, "textured.glsl", &[]);
			const W: f32 = 512.0;
			const H: f32 = 512.0;
			panels.push(PanelInstance {
				texture,
				shader,
				uv_x: &[0.0, 68.0 / W, 172.0 / W, 340.0 / W, 438.0 / W, 512.0 / W],
				uv_y: &[0.0, 62.0 / H, 170.0 / H, 328.0 / H, 434.0 / H, 512.0 / H],
				template_x: &[Abs(68.0), Fr(104.0), Abs(168.0), Fr(98.0), Abs(74.0)],
				template_y: &[Abs(62.0), Fr(108.0), Abs(158.0), Fr(106.0), Abs(78.0)],
				bounds: Bounds2(Point2(50, 50), Point2(600, 400)),
				min_size: Vec2(68 + 168 + 74, 62 + 158 + 78),
			});
		}

		{
			use d2::layout::Unit::*;
			let texture = {
				let bytes = assets.read("textures/panels.png").unwrap();
				let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
				g.image(&texture_props.bind(&image))
			};
			let shader = g.shader_compile(&mut shader_interface, "pixelart.glsl", &[]);
			const W: f32 = 64.0;
			const H: f32 = 256.0;
			const fn pos(x: i32, y: i32) -> Bounds2i {
				Bounds2i {
					mins: Vec2i { x: 128 + x * 148 - 16 * 4, y: 128 + y * 148 - 16 * 4 },
					maxs: Vec2i { x: 128 + x * 148 + 16 * 4, y: 128 + y * 148 + 16 * 4 },
				}
			}
			let specs = [
				(
					&[0.0 / W, 16.0 / W, 20.0 / W, 32.0 / W][..],
					&[0.0 / H, 12.0 / H, 18.0 / H, 32.0 / H][..],
					&[Abs(16.0 * 4.0), Fr(4.0 * 4.0), Abs(12.0 * 4.0)][..],
					&[Abs(12.0 * 4.0), Fr(6.0 * 4.0), Abs(14.0 * 4.0)][..],
					pos(0, 0),
					Vec2(112, 104),
				),
				(
					&[32.0 / W, 45.0 / W, 48.0 / W, 64.0 / W][..],
					&[0.0 / H, 13.0 / H, 19.0 / H, 32.0 / H][..],
					&[Abs(13.0 * 4.0), Fr(3.0 * 4.0), Abs(16.0 * 4.0)][..],
					&[Abs(13.0 * 4.0), Fr(6.0 * 4.0), Abs(13.0 * 4.0)][..],
					pos(1, 0),
					Vec2(116, 104),
				),
				(
					&[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W][..],
					&[32.0 / H, 38.0 / H, 58.0 / H, 64.0 / H][..],
					&[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)][..],
					&[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)][..],
					pos(0, 1),
					Vec2(48, 48),
				),
				(
					&[32.0 / W, 48.0 / W, 48.0 / W, 64.0 / W][..],
					&[32.0 / H, 48.0 / H, 48.0 / H, 64.0 / H][..],
					&[Abs(16.0 * 4.0), Fr(1.0), Abs(16.0 * 4.0)][..],
					&[Abs(16.0 * 4.0), Fr(1.0), Abs(16.0 * 4.0)][..],
					pos(1, 1),
					Vec2(128, 128),
				),
				(
					&[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W][..],
					&[64.0 / H, 75.0 / H, 88.0 / H, 96.0 / H][..],
					&[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)][..],
					&[Abs(11.0 * 4.0), Fr(13.0 * 4.0), Abs(8.0 * 4.0)][..],
					pos(0, 2),
					Vec2(48, 76),
				),
				(
					&[32.0 / W, 38.0 / W, 48.0 / W, 64.0 / W][..],
					&[64.0 / H, 70.0 / H, 84.0 / H, 96.0 / H][..],
					&[Abs(6.0 * 4.0), Fr(10.0 * 4.0), Abs(14.0 * 4.0)][..],
					&[Abs(6.0 * 4.0), Fr(14.0 * 4.0), Abs(12.0 * 4.0)][..],
					pos(1, 2),
					Vec2(80, 72),
				),
				(
					&[0.0 / W, 11.0 / W, 21.0 / W, 32.0 / W][..],
					&[96.0 / H, 107.0 / H, 117.0 / H, 128.0 / H][..],
					&[Abs(11.0 * 4.0), Fr(10.0 * 4.0), Abs(11.0 * 4.0)][..],
					&[Abs(11.0 * 4.0), Fr(10.0 * 4.0), Abs(11.0 * 4.0)][..],
					pos(0, 3),
					Vec2(88, 88),
				),
				(
					&[32.0 / W, 40.0 / W, 56.0 / W, 64.0 / W][..],
					&[96.0 / H, 104.0 / H, 120.0 / H, 128.0 / H][..],
					&[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)][..],
					&[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)][..],
					pos(1, 3),
					Vec2(64, 64),
				),
				(
					&[0.0 / W, 6.0 / W, 26.0 / W, 32.0 / W][..],
					&[128.0 / H, 135.0 / H, 153.0 / H, 160.0 / H][..],
					&[Abs(6.0 * 4.0), Fr(20.0 * 4.0), Abs(6.0 * 4.0)][..],
					&[Abs(7.0 * 4.0), Fr(18.0 * 4.0), Abs(7.0 * 4.0)][..],
					pos(2, 0),
					Vec2(48, 56),
				),
				(
					&[32.0 / W, 43.0 / W, 57.0 / W, 64.0 / W][..],
					&[128.0 / H, 135.0 / H, 149.0 / H, 160.0 / H][..],
					&[Abs(11.0 * 4.0), Fr(14.0 * 4.0), Abs(7.0 * 4.0)][..],
					&[Abs(7.0 * 4.0), Fr(14.0 * 4.0), Abs(11.0 * 4.0)][..],
					pos(2, 1),
					Vec2(72, 72),
				),
				(
					&[0.0 / W, 8.0 / W, 24.0 / W, 32.0 / W][..],
					&[160.0 / H, 169.0 / H, 183.0 / H, 192.0 / H][..],
					&[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)][..],
					&[Abs(9.0 * 4.0), Fr(14.0 * 4.0), Abs(9.0 * 4.0)][..],
					pos(2, 2),
					Vec2(64, 72),
				),
				(
					&[32.0 / W, 40.0 / W, 56.0 / W, 64.0 / W][..],
					&[160.0 / H, 165.0 / H, 187.0 / H, 192.0 / H][..],
					&[Abs(8.0 * 4.0), Fr(16.0 * 4.0), Abs(8.0 * 4.0)][..],
					&[Abs(5.0 * 4.0), Fr(22.0 * 4.0), Abs(5.0 * 4.0)][..],
					pos(2, 3),
					Vec2(64, 40),
				),
				(
					&[0.0 / W, 15.0 / W, 17.0 / W, 32.0 / W][..],
					&[192.0 / H, 202.0 / H, 214.0 / H, 224.0 / H][..],
					&[Abs(15.0 * 4.0), Fr(2.0 * 4.0), Abs(15.0 * 4.0)][..],
					&[Abs(10.0 * 4.0), Fr(12.0 * 4.0), Abs(10.0 * 4.0)][..],
					pos(3, 0),
					Vec2(120, 80),
				),
				(
					&[32.0 / W, 41.0 / W, 55.0 / W, 64.0 / W][..],
					&[192.0 / H, 201.0 / H, 215.0 / H, 224.0 / H][..],
					&[Abs(9.0 * 4.0), Fr(14.0 * 4.0), Abs(9.0 * 4.0)][..],
					&[Abs(10.0 * 4.0), Fr(12.0 * 4.0), Abs(10.0 * 4.0)][..],
					pos(3, 1),
					Vec2(72, 80),
				),
			];
			for (uv_x, uv_y, template_x, template_y, bounds, min_size) in specs {
				panels.push(PanelInstance {
					texture,
					shader,
					uv_x,
					uv_y,
					template_x,
					template_y,
					bounds,
					min_size,
				});
			}
		}

		let line_shader = g.shader_compile(&mut shader_interface, "color.glsl", &[]);
		Panels { panels, drag_state: None, cursor: Vec2i::ZERO, line_shader }
	}

	fn cursor_moved(&mut self, pos: Vec2i) {
		if let Some(drag_state) = &mut self.drag_state {
			let panel = &mut self.panels[drag_state.index];
			match drag_state.edge {
				DragEdge::Top => panel.bounds.mins.y = pos.y,
				DragEdge::Bottom => panel.bounds.maxs.y = pos.y,
				DragEdge::Left => panel.bounds.mins.x = pos.x,
				DragEdge::Right => panel.bounds.maxs.x = pos.x,
				DragEdge::Move => panel.bounds += pos - self.cursor,
				DragEdge::TopLeft => panel.bounds.mins = pos,
				DragEdge::TopRight => {
					panel.bounds.mins.y = pos.y;
					panel.bounds.maxs.x = pos.x;
				}
				DragEdge::BottomLeft => {
					panel.bounds.mins.x = pos.x;
					panel.bounds.maxs.y = pos.y;
				}
				DragEdge::BottomRight => panel.bounds.maxs = pos,
			}
			if panel.bounds.width() < panel.min_size.x {
				match drag_state.edge {
					DragEdge::Left | DragEdge::TopLeft | DragEdge::BottomLeft => panel.bounds.mins.x = panel.bounds.maxs.x - panel.min_size.x,
					DragEdge::Right | DragEdge::TopRight | DragEdge::BottomRight => panel.bounds.maxs.x = panel.bounds.mins.x + panel.min_size.x,
					_ => {}
				}
			}
			if panel.bounds.height() < panel.min_size.y {
				match drag_state.edge {
					DragEdge::Top | DragEdge::TopLeft | DragEdge::TopRight => panel.bounds.mins.y = panel.bounds.maxs.y - panel.min_size.y,
					DragEdge::Bottom | DragEdge::BottomLeft | DragEdge::BottomRight => panel.bounds.maxs.y = panel.bounds.mins.y + panel.min_size.y,
					_ => {}
				}
			}
		}
		self.cursor = pos;
	}

	fn hit_test(&self, pos: Vec2i) -> Option<(usize, DragEdge)> {
		for (i, panel) in self.panels.iter().enumerate().rev() {
			if let Some(edge) = panel.hit_test(pos) {
				return Some((i, edge));
			}
		}
		None
	}

	fn begin_drag(&mut self) -> bool {
		if let Some((index, edge)) = self.hit_test(self.cursor) {
			let last_index = self.panels.len() - 1;
			let tmp = self.panels.remove(index);
			self.panels.push(tmp);
			self.drag_state = Some(DragState { edge, index: last_index });
			return true;
		}
		false
	}

	fn end_drag(&mut self) {
		self.drag_state = None;
	}

	fn cursor(&self) -> Cursor {
		self.drag_state
			.as_ref()
			.map(|s| s.edge)
			.or_else(|| self.hit_test(self.cursor).map(|(_, edge)| edge))
			.map(DragEdge::cursor)
			.unwrap_or(Cursor::Default)
	}
}

impl DemoInterface for Panels {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position, .. } => {
				self.cursor_moved(Vec2(position.x as i32, position.y as i32));
				shell.set_cursor(self.cursor());
				shell.request_redraw();
			}
			Input::MouseButton { button: MouseButton::Left, pressed: true, .. } => {
				if self.begin_drag() {
					shell.set_cursor(self.cursor());
					shell.request_redraw();
				}
			}
			Input::MouseButton { button: MouseButton::Left, pressed: false, .. } => {
				self.end_drag();
				shell.set_cursor(self.cursor());
				shell.request_redraw();
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: Vec4(0.10, 0.11, 0.13, 1.0));
		let mut buf = d2::DrawBuilder::<d2::TexturedVertex, d2::TexturedUniform>::new();
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.cull_mode = None;
		buf.uniform.transform = Transform2::ortho(viewport.cast());
		for panel in &self.panels {
			buf.shader = panel.shader;
			buf.uniform.texture = panel.texture;
			let template_vertex = d2::TexturedTemplate { uv: Vec2::ZERO, color: Vec4(255, 255, 255, 255) };
			let pos_x = flex_panel(panel.bounds.left() as f32, panel.bounds.right() as f32, panel.template_x);
			let pos_y = flex_panel(panel.bounds.top() as f32, panel.bounds.bottom() as f32, panel.template_y);
			buf.panel_rect(&template_vertex, panel.uv_x, panel.uv_y, &pos_x, &pos_y);
		}
		buf.draw(g);

		if let Some(panel) = self.panels.last() {
			let mut buf = d2::DrawBuilder::<d2::ColorVertex, d2::ColorUniform>::new();
			buf.blend_mode = shade::BlendMode::Alpha;
			buf.cull_mode = None;
			buf.uniform.transform = Transform2::ortho(viewport.cast());
			buf.shader = self.line_shader;
			let color = Vec4(255, 255, 255, 128);
			let pen = d2::Pen {
				template: d2::ColorTemplate { color1: color, color2: color },
			};
			let pos_x = flex_panel(panel.bounds.left() as f32, panel.bounds.right() as f32, panel.template_x);
			let pos_y = flex_panel(panel.bounds.top() as f32, panel.bounds.bottom() as f32, panel.template_y);
			for x in pos_x {
				buf.draw_line(&pen, Point2(x, panel.bounds.top() as f32), Point2(x, panel.bounds.bottom() as f32));
			}
			for y in pos_y {
				buf.draw_line(&pen, Point2(panel.bounds.left() as f32, y), Point2(panel.bounds.right() as f32, y));
			}
			buf.draw(g);
		}
		g.end();
	}
}
