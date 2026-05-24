use super::*;

/// Split coordinates for a 9-slice grid.
///
/// * `x` is ordered left → right.
/// * `y` is ordered bottom → top.
///
/// ```text
/// y[3] +-----+-----------+-----+
///      |     |           |     |
/// y[2] +-----+-----------+-----+
///      |     | stretched |     |
/// y[1] +-----+-----------+-----+
///      |     |           |     |
/// y[0] +-----+-----------+-----+
///    x[0]  x[1]        x[2]  x[3]
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Panel {
	pub x: [f32; 4],
	pub y: [f32; 4],
}

impl<V: TVertex, U: TUniform> DrawBuilder<V, U> {
	/// Draws a 9-slice panel.
	///
	/// * `uv` defines source split lines.
	/// * `pos` defines destination split lines.
	///
	/// Produces a 4×4 vertex lattice:
	///
	/// ```text
	/// 12 --- 13 --- 14 --- 15
	///  |  /   |  /   |  /   |
	///  8 ---  9 --- 10 --- 11
	///  |  /   |  /   |  /   |
	///  4 ---  5 ---  6 ---  7
	///  |  /   |  /   |  /   |
	///  0 ---  1 ---  2 ---  3
	/// ```
	///
	/// Each cell is rendered as two triangles.
	#[inline(never)]
	pub fn panel_rect<T: ToVertexUV<V>>(&mut self, template: &T, uv: &Panel, pos: &Panel) {
		let mut cv = self.begin(PrimType::Triangles, 16, 18);
		for y in 0..3 {
			for x in 0..3 {
				let bottom_left = (y * 4 + x) as u32;
				let top_left = ((y + 1) * 4 + x) as u32;
				let top_right = ((y + 1) * 4 + x + 1) as u32;
				let bottom_right = (y * 4 + x + 1) as u32;
				cv.add_index3(bottom_left, top_left, top_right);
				cv.add_index3(bottom_left, top_right, bottom_right);
			}
		}

		for y in 0..4 {
			for x in 0..4 {
				let index = y * 4 + x;
				let pos = Point2(pos.x[x], pos.y[y]);
				let uv = Vec2(uv.x[x], uv.y[y]);
				let vertex = template.to_vertex_uv(pos, uv, index);
				cv.add_vertex(vertex);
			}
		}
	}
}
