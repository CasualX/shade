use super::*;

const SENSITIVITY: f32 = 0.01;

/// First-person camera for 3D navigation.
#[derive(Clone, Debug)]
pub struct FirstPersonCamera {
	/// Camera position.
	pub position: Vec3f,
	/// Forward direction the camera is facing.
	pub forward: Vec3f,
	/// Up direction relative to the camera.
	pub up: Vec3f,
}

impl FirstPersonCamera {
	pub fn new(position: Vec3f, forward: Vec3f, up: Vec3f) -> FirstPersonCamera {
		FirstPersonCamera { position, forward, up }
	}

	/// Moves the camera along its local axes.
	///
	/// The components of `dir` are treated as movement along (forward, right, up).
	pub fn r#move(&mut self, dir: Vec3f) {
		let right = self.forward.cross(self.up).norm();
		self.position += self.forward * dir.x + right * dir.y + self.up * dir.z;
	}

	/// Reorients the camera to look at a target point.
	pub fn look_at(&mut self, target: Vec3f) {
		self.forward = (target - self.position).norm();
	}

	/// Rotates the camera based on mouse movement.
	pub fn mouse(&mut self, dx: f32, dy: f32) {
		let yaw = Angle(dx * SENSITIVITY);
		let pitch = Angle(dy * SENSITIVITY);

		let forward = self.forward;
		let right = forward.cross(self.up).norm();
		let rotation = Mat3f::rotate(self.up, yaw) * Mat3f::rotate(right, pitch);

		self.forward = (rotation * forward).norm();
	}

	/// Returns the current camera position.
	#[inline]
	pub fn position(&self) -> Vec3f {
		self.position
	}

	/// Returns the view direction vector from the camera to the pivot point.
	#[inline]
	pub fn view_dir(&self) -> Vec3f {
		self.forward
	}

	/// Returns the view matrix using the given handedness.
	pub fn view_matrix(&self, hand: Hand) -> Transform3f {
		Transform3f::look_at(self.position, self.position + self.forward, self.up, hand)
	}
}
