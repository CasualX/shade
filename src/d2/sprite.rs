use super::*;

/// Sprite tool, places sprites.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite<T> {
	/// Vertex template for the bottom left vertex.
	pub bottom_left: T,
	/// Vertex template for the top left vertex.
	pub top_left: T,
	/// Vertex template for the top right vertex.
	pub top_right: T,
	/// Vertex template for the bottom right vertex.
	pub bottom_right: T,
}

impl<V: TVertex, U: TUniform> DrawBuilder<V, U> {
	/// Draws a sprite inside the given rectangle.
	#[inline(never)]
	pub fn sprite_rect<T: ToVertex<V>>(&mut self, sprite: &Sprite<T>, rc: &Bounds2f) {
		let vertices = [
			sprite.bottom_left.to_vertex(rc.bottom_left(), 0),
			sprite.top_left.to_vertex(rc.top_left(), 1),
			sprite.top_right.to_vertex(rc.top_right(), 2),
			sprite.bottom_right.to_vertex(rc.bottom_right(), 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}

	/// Draws a sprite with custom position, rotation, scale, or skew.
	///
	/// Unlike [`sprite_rect`](Self::sprite_rect), this method lets you place the sprite at any orientation or shape by providing a 2D transform.
	/// Conceptually, the sprite is a unit square from `0, 0` to `1, 1`, and each of its corners is transformed using the provided `pos`.
	/// The transform's translation sets the position of the bottom-left corner (at `0, 0`), while its X and Y axes define the direction and extent of the sprite's sides.
	///
	/// # Example
	///
	/// To position a sprite using three points:
	///
	/// ```
	/// use shade::d2;
	/// use shade::cvmath::{Vec2f, Vec4, Transform2f};
	///
	/// fn draw(buf: &mut d2::TexturedBuffer) {
	/// 	// Create a sprite with texture coordinates and no color modulation.
	/// 	let color = Vec4(255, 255, 255, 255);
	/// 	let sprite = d2::Sprite {
	/// 		bottom_left: d2::TexturedTemplate { uv: Vec2f(0.0, 0.0), color },
	/// 		top_left: d2::TexturedTemplate { uv: Vec2f(0.0, 1.0), color },
	/// 		top_right: d2::TexturedTemplate { uv: Vec2f(1.0, 1.0), color },
	/// 		bottom_right: d2::TexturedTemplate { uv: Vec2f(1.0, 0.0), color },
	/// 	};
	///
	/// 	let origin = Vec2f(100.0, 50.0);       // bottom-left corner
	/// 	let top_left = Vec2f(100.0, 100.0);    // defines Y axis
	/// 	let bottom_right = Vec2f(150.0, 50.0); // defines X axis
	///
	/// 	let x_axis = bottom_right - origin;
	/// 	let y_axis = top_left - origin;
	///
	/// 	let transform = Transform2f::compose(x_axis, y_axis, origin);
	/// 	buf.sprite_quad(&sprite, &transform);
	/// }
	/// ```
	#[inline(never)]
	pub fn sprite_quad<T: ToVertex<V>>(&mut self, sprite: &Sprite<T>, pos: &Transform2f) {
		let vertices = [
			sprite.bottom_left.to_vertex(pos.t(), 0),
			sprite.top_left.to_vertex(pos.t() + pos.y(), 1),
			sprite.top_right.to_vertex(pos.t() + pos.x() + pos.y(), 2),
			sprite.bottom_right.to_vertex(pos.t() + pos.x(), 3),
		];
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(&vertices);
	}
}
