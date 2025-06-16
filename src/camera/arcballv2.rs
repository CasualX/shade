use super::*;

const ROTATE_SPEED: f32 = 0.01;
const PAN_SPEED: f32 = 0.001;
const ZOOM_SPEED: f32 = 0.1;
const PITCH_LIMIT: Rad<f32> = Rad(90.0_f32.to_radians());

/// Arcball camera for 3D navigation.
#[derive(Clone, Debug)]
pub struct ArcballCamera {
	/// Point the camera orbits around and looks at.
	pub pivot: Vec3f,
	/// Distance from the camera to the pivot point.
	pub radius: f32,
	/// Pitch angle (up/down rotation).
	pub pitch: Rad<f32>,
	/// Yaw angle (left/right rotation).
	pub yaw: Rad<f32>,
	/// Axis used for pitch rotation.
	pub pitch_axis: Vec3f,
	/// Axis used for yaw rotation.
	pub yaw_axis: Vec3f,
}

impl ArcballCamera {
	/// Creates a new camera from a world position, pivot point, and up axis.
	///
	/// The distance between `position` and `pivot` must be greater than zero,
	/// and the view direction must not be aligned with the `up` axis.
	pub fn new(position: Vec3f, pivot: Vec3f, up: Vec3f) -> ArcballCamera {
		let offset = position - pivot;
		let (forward, radius) = offset.normalize_len();

		let yaw_axis = up.normalize();
		let pitch_axis = yaw_axis.cross(forward).normalize();

		let yaw = Rad(0.0); // Relative to the pitch axis
		let pitch = Rad(forward.dot(yaw_axis).asin());

		ArcballCamera { pivot, radius, yaw, pitch, yaw_axis, pitch_axis }
	}

	/// Rotates the camera around the pivot based on mouse delta.
	pub fn rotate(&mut self, dx: f32, dy: f32) {
		self.yaw += Rad(dx * ROTATE_SPEED);
		self.pitch += Rad(dy * ROTATE_SPEED);

		// Unclamped pitch can work, but the view_matrix() code below needs to be patched to use proper up vector
		self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
	}

	/// Pans the camera parallel to the view plane.
	pub fn pan(&mut self, dx: f32, dy: f32) {
		let rotation = Mat3f::rotate(self.yaw, self.yaw_axis) * Mat3f::rotate(self.pitch, self.pitch_axis);
		let right = rotation * self.pitch_axis;
		let up = rotation * self.yaw_axis;

		let pan_speed = self.radius * PAN_SPEED;
		let pan = right * (dx * pan_speed) + up * (dy * pan_speed);
		self.pivot += pan;
	}

	/// Zooms the camera based on the scale factor.
	pub fn zoom(&mut self, scale: f32) {
		self.radius *= 1.0 - scale * ZOOM_SPEED;
		self.radius = f32::max(self.radius, 0.01);
	}

	/// Returns the current camera position.
	pub fn position(&self) -> Vec3f {
		self.pivot - self.view_dir() * self.radius
	}

	/// Returns the view direction vector from the camera to the pivot point.
	pub fn view_dir(&self) -> Vec3f {
		let forward = self.yaw_axis.cross(self.pitch_axis).normalize();
		Mat3f::rotate(self.yaw, self.yaw_axis) * Mat3f::rotate(self.pitch, self.pitch_axis) * forward
	}

	/// Returns the view matrix using the given handedness.
	pub fn view_matrix(&self, hand: Hand) -> Mat4f {
		// let up = self.yaw_axis; // Use the yaw axis as the up vector
		let rotation = Mat3f::rotate(self.yaw, self.yaw_axis) * Mat3f::rotate(self.pitch, self.pitch_axis);
		let up = rotation * self.yaw_axis;
		Mat4f::look_at(self.position(), self.pivot, up, hand)
	}
}
