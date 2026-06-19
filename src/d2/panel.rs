use super::*;

/// Fixed-size split lines for a subdivided rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Panel<const NX: usize, const NY: usize = NX> {
	pub x: [f32; NX],
	pub y: [f32; NY],
}

impl<'a, V: TVertex, U: TUniform> DrawBuilder<'a, V, U> {
	/// Draws a subdivided panel.
	///
	/// * `uv_x` and `uv_y` define source split lines.
	/// * `pos_x` and `pos_y` define destination split lines.
	///
	/// Each cell is rendered as two triangles.
	#[inline(never)]
	pub fn panel_rect<T: ToVertexUV<V>>(&mut self, template: &T, uv_x: &[f32], uv_y: &[f32], pos_x: &[f32], pos_y: &[f32]) {
		assert_eq!(uv_x.len(), pos_x.len());
		assert_eq!(uv_y.len(), pos_y.len());
		if uv_x.len() < 2 || uv_y.len() < 2 {
			return;
		}

		let width = uv_x.len();
		let height = uv_y.len();
		let cells_x = width - 1;
		let cells_y = height - 1;

		let mut cv = self.begin(PrimType::Triangles, width * height, cells_x * cells_y * 2);
		for y in 0..cells_y {
			for x in 0..cells_x {
				let bottom_left = (y * width + x) as u32;
				let top_left = ((y + 1) * width + x) as u32;
				let top_right = ((y + 1) * width + x + 1) as u32;
				let bottom_right = (y * width + x + 1) as u32;
				cv.add_index3(bottom_left, top_left, top_right);
				cv.add_index3(bottom_left, top_right, bottom_right);
			}
		}

		for y in 0..height {
			for x in 0..width {
				let index = y * width + x;
				let pos = Point2(pos_x[x], pos_y[y]);
				let uv = Vec2(uv_x[x], uv_y[y]);
				let vertex = template.to_vertex_uv(pos, uv, index);
				cv.add_vertex(vertex);
			}
		}
	}

	/// Draws a subdivided panel using fixed-size split lines.
	///
	/// See [`panel_rect`](Self::panel_rect) for details.
	#[inline]
	pub fn panel_rect_n<T: ToVertexUV<V>, const NX: usize, const NY: usize>(&mut self, template: &T, uv: &Panel<NX, NY>, pos: &Panel<NX, NY>) {
		self.panel_rect(template, &uv.x, &uv.y, &pos.x, &pos.y);
	}
}
