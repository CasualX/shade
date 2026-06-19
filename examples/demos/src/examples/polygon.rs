use crate::*;

const PICK_RADIUS_PX: f32 = 12.0;
const NO_DRAG: usize = usize::MAX;

pub fn create(g: &mut shade::Graphics, _assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Polygon::new(g))
}

struct Polygon {
	color_shader: Box<dyn shade::ShaderProgram>,
	points: Vec<Point2f>,
	cursor_px: Vec2f,
	view_offset: Vec2f,
	panning: bool,
	pan_start_px: Vec2f,
	dragging: usize,
	shift: bool,
}

impl Polygon {
	fn new(g: &mut shade::Graphics) -> Polygon {
		let mut shader_source = shade::shader_interface! {
			files {
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let color_shader = g.shader_compile(&mut shader_source, "color.glsl", &[]);
		Polygon {
			color_shader,
			points: Vec::new(),
			cursor_px: Vec2f::ZERO,
			view_offset: Vec2f::ZERO,
			panning: false,
			pan_start_px: Vec2f::ZERO,
			dragging: NO_DRAG,
			shift: false,
		}
	}

	fn cursor_world(&self) -> Point2f {
		Point2(self.cursor_px.x + self.view_offset.x, self.cursor_px.y + self.view_offset.y)
	}

	fn pick_point_index(&self) -> Option<usize> {
		let mut best: Option<(usize, f32)> = None;
		let r2 = PICK_RADIUS_PX * PICK_RADIUS_PX;
		let cursor = self.cursor_world();
		for (i, p) in self.points.iter().enumerate() {
			let dist2 = p.distance_sqr(cursor);
			if dist2 <= r2 {
				match best {
					None => best = Some((i, dist2)),
					Some((_, best2)) if dist2 < best2 => best = Some((i, dist2)),
					_ => {}
				}
			}
		}
		best.map(|(i, _)| i)
	}

	fn start_drag_index(&mut self, idx: usize) {
		self.dragging = idx;
	}

	fn begin_drag(&mut self) -> bool {
		if self.dragging != NO_DRAG {
			return true;
		}
		if let Some(i) = self.pick_point_index() {
			self.start_drag_index(i);
			return true;
		}
		false
	}

	fn pick_edge_insertion(&self) -> Option<(usize, Point2f)> {
		let n = self.points.len();
		if n < 2 {
			return None;
		}
		let close = n >= 3;
		let mut best: Option<(usize, Point2f, f32)> = None;
		let cursor = self.cursor_world();
		let seg_count = if close { n } else { n - 1 };
		for i in 0..seg_count {
			let seg = Line2(self.points[i], self.points[(i + 1) % n]);
			let dist = seg.distance(cursor);
			let q = seg.project(cursor);
			let insert_index = if close && i == n - 1 { n } else { i + 1 };
			let dist2 = dist * dist;
			match best {
				None => best = Some((insert_index, q, dist2)),
				Some((_, _, best2)) if dist2 < best2 => best = Some((insert_index, q, dist2)),
				_ => {}
			}
		}
		best.map(|(idx, q, _)| (idx, q))
	}

	fn split_edge_at_cursor(&mut self) -> usize {
		let (insert_index, q) = self.pick_edge_insertion().expect("split_edge_at_cursor requires at least 2 points");
		self.points.insert(insert_index, q);
		insert_index
	}

	fn end_drag(&mut self) {
		self.dragging = NO_DRAG;
	}

	fn drag_to_cursor(&mut self) {
		let i = self.dragging;
		if i == NO_DRAG {
			return;
		}
		if i >= self.points.len() {
			self.end_drag();
			return;
		}
		self.points[i] = self.cursor_world();
	}

	fn delete_point_at_cursor(&mut self) -> bool {
		let Some(idx) = self.pick_point_index() else {
			return false;
		};
		if self.dragging != NO_DRAG {
			if self.dragging == idx {
				self.end_drag();
			}
			else if idx < self.dragging {
				self.dragging -= 1;
			}
		}
		self.points.remove(idx);
		true
	}

	fn add_point_from_cursor(&mut self) -> usize {
		self.points.push(self.cursor_world());
		self.points.len() - 1
	}
}

impl DemoInterface for Polygon {
	fn redraw_mode(&self) -> RedrawMode {
		RedrawMode::OnDemand
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
		match input {
			Input::KeyDown(Key::Shift) => self.shift = true,
			Input::KeyUp(Key::Shift) => self.shift = false,
			Input::MouseMove { position, .. } => {
				self.cursor_px = position;
				if self.panning {
					let delta = self.cursor_px - self.pan_start_px;
					self.view_offset -= delta;
					self.pan_start_px = self.cursor_px;
					shell.request_redraw();
				}
				else if self.dragging != NO_DRAG {
					self.drag_to_cursor();
					shell.request_redraw();
				}
			}
			Input::MouseButton { button: MouseButton::Left, pressed: true, .. } => {
				if self.shift {
					self.panning = true;
					self.pan_start_px = self.cursor_px;
					self.end_drag();
					shell.request_redraw();
					return;
				}
				if !self.begin_drag() {
					let idx = if self.points.len() < 2 {
						self.add_point_from_cursor()
					}
					else {
						self.split_edge_at_cursor()
					};
					self.start_drag_index(idx);
				}
				self.drag_to_cursor();
				shell.request_redraw();
			}
			Input::MouseButton { button: MouseButton::Left, pressed: false, .. } => {
				self.panning = false;
				self.end_drag();
			}
			Input::MouseButton { button: MouseButton::Right, pressed: true, .. } => {
				if self.delete_point_at_cursor() {
					shell.request_redraw();
				}
			}
			_ => {}
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.08, 0.09, 0.11, 1.0));
		let mut cv = shade::d2::ColorBuffer::new();
		cv.shader = Some(&*self.color_shader);
		cv.blend_mode = shade::BlendMode::Alpha;
		let w = frame.viewport.width() as f32;
		let h = frame.viewport.height() as f32;
		let world = Bounds2!(self.view_offset.x, self.view_offset.y, self.view_offset.x + w, self.view_offset.y + h);
		cv.uniform.transform = Transform2::ortho(world);
		let fill_paint = shade::d2::Paint {
			template: shade::d2::ColorTemplate { color1: Vec4(40, 180, 255, 120), color2: Vec4(40, 180, 255, 120) },
		};
		let outline_pen = shade::d2::Pen {
			template: shade::d2::ColorTemplate { color1: Vec4(240, 240, 240, 220), color2: Vec4(240, 240, 240, 220) },
		};
		let point_paint = shade::d2::Paint {
			template: shade::d2::ColorTemplate { color1: Vec4(255, 210, 0, 255), color2: Vec4(255, 210, 0, 255) },
		};
		let tri_pen = shade::d2::Pen {
			template: shade::d2::ColorTemplate { color1: Vec4(255, 255, 255, 80), color2: Vec4(255, 255, 255, 80) },
		};

		if self.points.len() >= 3 {
			let tris = shade::d2::polygon::triangulate(&self.points);
			cv.fill_polygon(&fill_paint, &self.points, &tris);
			let mut edges: HashSet<(u32, u32)> = HashSet::new();
			for &shade::Index3 { p1, p2, p3 } in &tris {
				let (ab0, ab1) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
				let (bc0, bc1) = if p2 < p3 { (p2, p3) } else { (p3, p2) };
				let (ca0, ca1) = if p3 < p1 { (p3, p1) } else { (p1, p3) };
				edges.insert((ab0, ab1));
				edges.insert((bc0, bc1));
				edges.insert((ca0, ca1));
			}
			let lines: Vec<(u32, u32)> = edges.into_iter().collect();
			cv.draw_lines(&tri_pen, &self.points, &lines);
		}
		if self.points.len() >= 2 {
			cv.draw_poly_line(&outline_pen, &self.points, self.points.len() >= 3);
		}
		for p in &self.points {
			let r = 4.0;
			cv.fill_ellipse(&point_paint, &Bounds2!(p.x - r, p.y - r, p.x + r, p.y + r), 12);
		}
		cv.draw(g);
		g.end();
	}
}
