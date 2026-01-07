use super::*;

const PAN_SPEED: f32 = 0.001;

/// Planar camera for 3D navigation.
///
/// This camera keeps a fixed orientation basis provided by the caller:
/// - `right` is local +X (towards the right of the screen)
/// - `forward` is local +Y (towards the top of the screen)
/// - `view_dir` is the look direction
///
/// Movement is expressed in that basis via [`PlanarCamera::pan`].
#[derive(Clone, Debug)]
pub struct PlanarCamera {
	/// Camera position.
	pub position: Vec3f,
	/// View direction the camera is facing.
	pub view_dir: Vec3f,
	/// Local +X movement axis.
	pub right: Vec3f,
	/// Local +Y movement axis (also used as the view-matrix up vector).
	pub forward: Vec3f,
}

impl PlanarCamera {
	/// Creates a new planar camera.
	pub fn new(position: Vec3f, view_dir: Vec3f, right: Vec3f, forward: Vec3f) -> PlanarCamera {
		PlanarCamera { position, view_dir: view_dir.norm(), right: right.norm(), forward: forward.norm() }
	}

	/// Moves the camera in its local basis.
	pub fn pan(&mut self, dir: Vec3f) {
		let up = self.forward.cross(self.right).norm();
		self.position += self.right * (dir.x * PAN_SPEED) + self.forward * (dir.y * PAN_SPEED) + up * (dir.z * PAN_SPEED);
	}

	/// Returns the current camera position.
	#[inline]
	pub fn position(&self) -> Vec3f {
		self.position
	}

	/// Returns the view direction vector.
	#[inline]
	pub fn view_dir(&self) -> Vec3f {
		self.view_dir
	}

	/// Returns the view matrix using the given handedness.
	pub fn view_matrix(&self, hand: Hand) -> Transform3f {
		Transform3f::look_at(self.position, self.position + self.view_dir, self.forward, hand)
	}
}
