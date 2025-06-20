/*!
3D cameras.
*/

use cvmath::*;
use super::*;

mod arcballv2;
pub use self::arcballv2::ArcballCamera;

mod firstperson;
pub use self::firstperson::FirstPersonCamera;

/// Contains camera matrices and parameters.
#[derive(Clone, Debug)]
pub struct CameraSetup {
	pub viewport: Bounds2<i32>,
	pub aspect_ratio: f32,
	pub position: Vec3f,
	pub near: f32,
	pub far: f32,
	pub view: Mat4f,
	pub projection: Mat4f,
	pub view_proj: Mat4f,
	pub inv_view_proj: Mat4f,
	pub clip: Clip,
}

impl CameraSetup {
	/// Projects a 3D world-space point to 2D screen-space (in pixels), or returns None if behind camera.
	pub fn world_to_screen(&self, pt: Vec3f) -> Option<Vec2f> {
		// Transform world-space point into clip space
		let clip = self.view_proj * pt.vec4(1.0);

		// Reject points behind the camera
		if clip.w.abs() < f32::EPSILON {
			return None;
		}

		let ndc = clip.hdiv();

		// Map NDC to screen-space coordinates
		let x = (ndc.x + 1.0) * 0.5 * self.viewport.width() as f32 + self.viewport.mins.x as f32;
		let y = (1.0 - ndc.y) * 0.5 * self.viewport.height() as f32 + self.viewport.mins.y as f32; // Flip Y

		Some(Vec2f(x, y))
	}

	/// Generates a world-space ray from a screen-space pixel coordinate.
	pub fn unproject(&self, pt: Vec2i) -> Ray<f32> {
		// Convert screen pixel to normalized device coordinates (NDC)
		let ndc_x = 2.0 * (pt.x as f32 - self.viewport.mins.x as f32) / (self.viewport.width() as f32) - 1.0;
		let ndc_y = 1.0 - 2.0 * (pt.y as f32 - self.viewport.mins.y as f32) / (self.viewport.height() as f32); // Flip Y!

		// Construct clip-space positions at near and far depth
		let ndc_near = match self.clip { Clip::NO => -1.0, Clip::ZO => 0.0 };
		let ndc_far = 1.0;
		let near_clip = Vec4f(ndc_x, ndc_y, ndc_near, 1.0);
		let far_clip  = Vec4f(ndc_x, ndc_y, ndc_far, 1.0);

		// Transform to world-space using the inverse view-projection matrix
		let near = (self.inv_view_proj * near_clip).hdiv();
		let far  = (self.inv_view_proj * far_clip).hdiv();

		// Create a ray from near to far
		let direction = (far - near).normalize();
		Ray { origin: near, direction }
	}
}

impl UniformVisitor for CameraSetup {
	fn visit(&self, set: &mut dyn UniformSetter) {
		set.value("u_viewport", &self.viewport);
		set.value("u_aspect_ratio", &self.aspect_ratio);
		set.value("u_position", &self.position);
		set.value("u_near", &self.near);
		set.value("u_far", &self.far);
		set.value("u_view", &self.view);
		set.value("u_projection", &self.projection);
		set.value("u_view_proj", &self.view_proj);
		set.value("u_inv_view_proj", &self.inv_view_proj);
	}
}
