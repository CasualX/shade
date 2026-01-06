//! Graphics module 3D.

use super::*;
use cvmath::*;

mod camera;
pub use self::camera::*;

mod textured;
pub use self::textured::*;

mod color3d;
pub use self::color3d::*;

mod mesh;
pub use self::mesh::*;

pub mod submesh;

pub mod axes;
pub mod frustum;
pub mod icosahedron;

/// Computes the axis-aligned bounding box for the given vertices.
#[inline]
pub fn compute_bounds<V: TVertex3>(vertices: &[V]) -> Bounds3f {
	if vertices.is_empty() {
		return Bounds3f::ZERO;
	}
	let mut bounds = Bounds3f(Vec3f::dup(f32::MAX), Vec3f::dup(f32::MIN));
	for v in vertices {
		let pos = v.position();
		bounds = bounds.include(pos);
	}
	return bounds;
}
