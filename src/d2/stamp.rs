use super::*;

/// Stamp prints sprites.
#[derive(Clone, Debug, PartialEq)]
pub struct Stamp<T> {
	/// Vertex template for the bottom left vertex.
	pub bottom_left: T,
	/// Vertex template for the top left vertex.
	pub top_left: T,
	/// Vertex template for the top right vertex.
	pub top_right: T,
	/// Vertex template for the bottom right vertex.
	pub bottom_right: T,
}

impl<V: TVertex, U: TUniform> CommandBuffer<V, U> {
	#[inline(never)]
	pub fn stamp_rect<T: ToVertex<V>>(&mut self, stamp: &Stamp<T>, rc: &Rect<f32>) {
		let vertices = [
			stamp.bottom_left.to_vertex(rc.bottom_left(), 0),
			stamp.top_left.to_vertex(rc.top_left(), 1),
			stamp.top_right.to_vertex(rc.top_right(), 2),
			stamp.bottom_right.to_vertex(rc.bottom_right(), 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}

	#[inline(never)]
	pub fn stamp_quad<T: ToVertex<V>>(&mut self, stamp: &Stamp<T>, pos: &Transform2<f32>) {
		let vertices = [
			stamp.bottom_left.to_vertex(pos.t(), 0),
			stamp.top_left.to_vertex(pos.t() + pos.y(), 1),
			stamp.top_right.to_vertex(pos.t() + pos.x() + pos.y(), 2),
			stamp.bottom_right.to_vertex(pos.t() + pos.x(), 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}
}
