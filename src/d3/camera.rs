/*!
3D cameras.
*/

use cvmath::*;
use super::*;

mod arcball;
pub use self::arcball::ArcballCamera;

mod firstperson;
pub use self::firstperson::FirstPersonCamera;

mod planar;
pub use self::planar::PlanarCamera;

/// Contains camera matrices and parameters.
#[derive(Clone, Debug)]
pub struct Camera {
	pub viewport: Bounds2<i32>,
	pub aspect_ratio: f32,
	pub position: Vec3f,
	pub view: Transform3f,
	pub near: f32,
	pub far: f32,
	pub projection: Mat4f,
	pub view_proj: Mat4f,
	pub inv_view_proj: Mat4f,
	pub clip: Clip,
}

impl Camera {
	/// Projects a 3D world-space point into 2D viewport-space (in pixels), or returns None if behind the camera.
	pub fn world_to_viewport(&self, pt: Vec3f) -> Option<Vec2f> {
		// Transform world-space point into clip space
		let clip = self.view_proj * pt.vec4(1.0);

		// Reject points behind the camera
		if clip.w < f32::EPSILON {
			return None;
		}

		let ndc = clip.hdiv();

		// Map NDC to screen-space coordinates
		let x = (ndc.x + 1.0) * 0.5 * self.viewport.width() as f32 + self.viewport.mins.x as f32;
		let y = (1.0 - ndc.y) * 0.5 * self.viewport.height() as f32 + self.viewport.mins.y as f32; // Flip Y

		Some(Vec2f(x, y))
	}

	/// Generates a world-space ray from a 2D viewport-space pixel coordinate.
	pub fn viewport_to_ray(&self, pt: Vec2i) -> Ray3<f32> {
		// Convert screen pixel to normalized device coordinates (NDC)
		let ndc_x = 2.0 * (pt.x - self.viewport.mins.x) as f32 / (self.viewport.width() as f32) - 1.0;
		let ndc_y = 1.0 - 2.0 * (pt.y - self.viewport.mins.y) as f32 / (self.viewport.height() as f32); // Flip Y!

		// Construct clip-space positions at near and far depth
		let ndc_near = match self.clip { Clip::NO => -1.0, Clip::ZO => 0.0 };
		let ndc_far = 1.0;
		let near_clip = Vec4f(ndc_x, ndc_y, ndc_near, 1.0);
		let far_clip = Vec4f(ndc_x, ndc_y, ndc_far, 1.0);

		// Transform to world-space using the inverse view-projection matrix
		let near = (self.inv_view_proj * near_clip).hdiv();
		let far = (self.inv_view_proj * far_clip).hdiv();

		// Create a ray from near to far
		let direction = (far - near).norm();
		Ray3 { origin: near, direction, distance: cvmath::Interval(0.0, f32::INFINITY) }
	}

	/// Returns true if the given bounds intersect the camera's view frustum.
	///
	/// If the bounds are in local space, provide the transform from local to world space.
	pub fn is_visible(&self, bounds: &Bounds3f, bounds_transform: Option<&Transform3f>) -> bool {
		let transform = if let Some(&bounds_transform) = bounds_transform {
			self.view_proj * bounds_transform
		}
		else {
			self.view_proj
		};

		const INDICES: [[bool; 3]; 8] = [
			[false, false, false],
			[false, false, true ],
			[false, true,  false],
			[false, true,  true ],
			[true,  false, false],
			[true,  false, true ],
			[true,  true,  false],
			[true,  true,  true ],
		];

		// Transform to clip space
		let &pts: &[Vec3f; 2] = bounds.as_ref();
		let clip_pts = INDICES.map(|[x, y, z]| transform * Vec4f(pts[x as usize].x, pts[y as usize].y, pts[z as usize].z, 1.0));

		// Perform frustum culling using the clip space coordinates
		if clip_pts.iter().all(|p| p.x < -p.w) { return false }
		if clip_pts.iter().all(|p| p.x >  p.w) { return false }
		if clip_pts.iter().all(|p| p.y < -p.w) { return false }
		if clip_pts.iter().all(|p| p.y >  p.w) { return false }
		match self.clip {
			Clip::NO => if clip_pts.iter().all(|p| p.z < -p.w) { return false },
			Clip::ZO => if clip_pts.iter().all(|p| p.z < 0.0) { return false },
		}
		if clip_pts.iter().all(|p| p.z > p.w) { return false }

		return true;
	}
}

impl UniformVisitor for Camera {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_viewport", &self.viewport);
		set.value("u_aspectRatio", &self.aspect_ratio);
		set.value("u_cameraPosition", &self.position);
		set.value("u_zNear", &self.near);
		set.value("u_zFar", &self.far);
		set.value("u_viewMatrix", &self.view);
		set.value("u_projMatrix", &self.projection);
		set.value("u_viewProjMatrix", &self.view_proj);
		set.value("u_invViewProjMatrix", &self.inv_view_proj);

		let ndc_near: f32 = match self.clip { Clip::NO => -1.0, Clip::ZO => 0.0 };
		set.value("u_ndcNear", &ndc_near);
	}
}

#[cfg(test)]
mod tests;
