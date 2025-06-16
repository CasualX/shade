use super::*;

const ROTATE_SPEED: f32 = 0.01;
const PAN_SPEED: f32 = 0.001;
const ZOOM_SPEED: f32 = 0.1;

pub struct ArcballCamera {
	pub target: Vec3f,
	pub position: Vec3f,
	pub up: Vec3f,
}

impl ArcballCamera {
	#[inline]
	pub fn new(position: Vec3f, target: Vec3f, up: Vec3f) -> ArcballCamera {
		ArcballCamera { target, position, up }
	}

	pub fn rotate(&mut self, dx: f32, dy: f32) {
		let yaw = cvmath::Rad(dx * ROTATE_SPEED);
		let pitch = cvmath::Rad(dy * ROTATE_SPEED);

		let forward = self.target - self.position;
		let right = forward.cross(self.up).normalize();
		let rotation = Mat3f::rotate(yaw, self.up) * Mat3f::rotate(pitch, right);

		self.position = self.target + rotation * (self.position - self.target);
	}

	pub fn pan(&mut self, dx: f32, dy: f32) {
		let forward = self.target - self.position;
		let pan_speed = forward.len() * PAN_SPEED;

		let right = forward.cross(self.up).normalize();
		let up = right.cross(forward).normalize();

		let pan = right * (dx * pan_speed) + up * (dy * pan_speed);

		self.position += pan;
		self.target += pan;
	}

	pub fn zoom(&mut self, scale: f32) {
		let forward = self.target - self.position;
		self.position += forward * (scale * ZOOM_SPEED);
	}

	pub fn snap_to(&mut self, dir: Vec3f) {
		let radius = (self.target - self.position).len();
		self.position = self.target - dir.normalize() * radius;
	}

	/// Returns the camera position.
	#[inline]
	pub fn position(&self) -> Vec3f {
		self.position
	}

	/// Returns a view matrix for this camera using the given handedness.
	#[inline]
	pub fn view_matrix(&self, hand: Hand) -> Mat4f {
		Mat4f::look_at(self.position, self.target, self.up, hand)
	}
}

