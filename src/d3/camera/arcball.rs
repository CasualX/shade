use super::*;

const ROTATE_SPEED: f32 = 0.01;
const PAN_SPEED: f32 = 0.001;
const ZOOM_SPEED: f32 = 0.1;
const PITCH_LIMIT: Angle<f32> = Angle(90.0_f32.to_radians());

/// Arcball camera for 3D navigation.
#[derive(Clone, Debug)]
pub struct ArcballCamera {
	/// Point the camera orbits around and looks at.
	pub pivot: Vec3f,
	/// Distance from the camera to the pivot point.
	pub radius: f32,
	/// Pitch angle (up/down rotation).
	pub pitch: Angle<f32>,
	/// Yaw angle (left/right rotation).
	pub yaw: Angle<f32>,
	/// Axis used for pitch rotation.
	pub pitch_axis: Vec3f,
	/// Axis used for yaw rotation.
	pub yaw_axis: Vec3f,
}

impl ArcballCamera {
	/// Creates a new camera from a world position, pivot point, and reference up axis.
	///
	/// The distance between `position` and `pivot` must be greater than zero,
	/// and the view direction must not be aligned with the `ref_up` axis.
	pub fn new(position: Vec3f, pivot: Vec3f, ref_up: Vec3f) -> ArcballCamera {
		let offset = pivot - position;
		let (forward, radius) = offset.norm_len();

		let yaw_axis = ref_up.norm();
		let pitch_axis = forward.cross(yaw_axis).norm();

		let yaw = Angle(0.0); // Relative to the pitch axis
		let pitch = Angle(forward.dot(yaw_axis).asin());

		ArcballCamera { pivot, radius, yaw, pitch, yaw_axis, pitch_axis }
	}

	/// Rotates the camera around the pivot based on mouse delta.
	pub fn rotate(&mut self, dx: f32, dy: f32) {
		self.yaw += Angle(dx * ROTATE_SPEED);
		self.pitch += Angle(dy * ROTATE_SPEED);

		// Clamp pitch for UX reasons
		self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
	}

	/// Pans the camera parallel to the view plane.
	pub fn pan(&mut self, dx: f32, dy: f32) {
		let rotation = Mat3f::rotate(self.yaw_axis, self.yaw) * Mat3f::rotate(self.pitch_axis, self.pitch);
		let right = rotation * self.pitch_axis;
		let up = rotation * self.yaw_axis;

		let pan_speed = self.radius * PAN_SPEED;
		let pan = right * (dx * pan_speed) + up * (dy * pan_speed);
		self.pivot += pan;
	}

	/// Zooms the camera based on the scale factor.
	pub fn zoom(&mut self, scale: f32) {
		self.radius *= 1.0 - scale * ZOOM_SPEED;
		// self.radius = f32::max(self.radius, 0.01);
	}

	/// Returns the current camera position.
	pub fn position(&self) -> Vec3f {
		self.pivot - self.view_dir() * self.radius
	}

	/// Returns the view direction vector from the camera to the pivot point.
	pub fn view_dir(&self) -> Vec3f {
		let forward = self.yaw_axis.cross(self.pitch_axis).norm();
		Mat3f::rotate(self.yaw_axis, self.yaw) * Mat3f::rotate(self.pitch_axis, self.pitch) * forward
	}

	/// Returns the view matrix using the given handedness.
	pub fn view_matrix(&self, hand: Hand) -> Transform3f {
		// Compute the proper up vector based on the yaw and pitch axes
		// It's cheaper to just use the yaw axis, but this breaks when looking straight up or down
		let up = Mat3f::rotate(self.yaw_axis, self.yaw) * Mat3f::rotate(self.pitch_axis, self.pitch) * self.yaw_axis;
		Transform3f::look_at(self.position(), self.pivot, up, hand)
	}
}
