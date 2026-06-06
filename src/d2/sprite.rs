use super::*;

/// Sprite tool, places sprites.
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
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

impl<T> Sprite<T> {
	/// Permutes the vertices of the sprite according to the given transform.
	pub fn transform(&self, transform: atlas::Transform) -> Sprite<T> where T: Copy {
		let indices = match transform {
			atlas::Transform::None => [0, 1, 2, 3],
			atlas::Transform::Rotate90 => [3, 0, 1, 2],
			atlas::Transform::Rotate180 => [2, 3, 0, 1],
			atlas::Transform::Rotate270 => [1, 2, 3, 0],
			atlas::Transform::FlipX => [3, 2, 1, 0],
			atlas::Transform::FlipY => [1, 0, 3, 2],
			atlas::Transform::FlipSlash => [0, 3, 2, 1],
			atlas::Transform::FlipBackslash => [2, 1, 0, 3],
		};
		let vertices = [self.bottom_left, self.top_left, self.top_right, self.bottom_right];
		Sprite {
			bottom_left: vertices[indices[0]],
			top_left: vertices[indices[1]],
			top_right: vertices[indices[2]],
			bottom_right: vertices[indices[3]],
		}
	}
}

impl<T> AsRef<[T; 4]> for Sprite<T> {
	#[inline]
	fn as_ref(&self) -> &[T; 4] {
		// Sprite is repr(C) and contains exactly these four T fields in order.
		unsafe { mem::transmute(self) }
	}
}

impl<'a, V: TVertex, U: TUniform> DrawBuilder<'a, V, U> {
	/// Draws a sprite inside the given rectangle.
	#[inline(never)]
	pub fn sprite_rect<T: ToVertex<V>>(&mut self, sprite: &Sprite<T>, rc: &Bounds2f) {
		let vertices = Sprite {
			bottom_left: sprite.bottom_left.to_vertex(rc.bottom_left(), 0),
			top_left: sprite.top_left.to_vertex(rc.top_left(), 1),
			top_right: sprite.top_right.to_vertex(rc.top_right(), 2),
			bottom_right: sprite.bottom_right.to_vertex(rc.bottom_right(), 3),
		};
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(vertices.as_ref());
	}

	/// Draws a sprite with custom position, rotation, scale, or skew.
	///
	/// Unlike [sprite_rect](Self::sprite_rect), this method lets you place the sprite at any orientation or shape by providing a 2D transform.
	/// Conceptually, the sprite is a unit square from `0, 0` to `1, 1`, and each of its vertices is transformed using the provided `pos`.
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
		let vertices = Sprite {
			bottom_left: sprite.bottom_left.to_vertex(pos.t(), 0),
			top_left: sprite.top_left.to_vertex(pos.t() + pos.y(), 1),
			top_right: sprite.top_right.to_vertex(pos.t() + pos.x() + pos.y(), 2),
			bottom_right: sprite.bottom_right.to_vertex(pos.t() + pos.x(), 3),
		};
		let mut cv = self.begin(PrimType::Triangles, 4, 2);
		cv.add_indices_quad();
		cv.add_vertices(vertices.as_ref());
	}
}
