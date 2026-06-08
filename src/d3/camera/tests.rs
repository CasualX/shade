use super::*;

#[test]
fn test_camera_setup() {
	let viewport = Bounds2!(0, 0, 100, 100);
	let aspect_ratio = 1.0;
	let fov_y = Angle::deg(90.0);
	let (near, far) = (1.0, 10.0);

	let position = Vec3(5.0, 0.0, 0.0);
	let target = Vec3::ZERO;
	let forward = (target - position).norm();
	let up = Vec3::Y;

	println!("Viewport: {viewport:?}");
	println!("Aspect ratio: {aspect_ratio}");
	println!("FOV (Y): {fov_y}");
	println!("Near: {near}");
	println!("Far: {far}");
	println!("Camera position: {position:?}");
	println!("Camera target: {target:?}");
	println!("Camera forward: {forward:?}");
	println!("Camera up: {up:?}");

	let view = Transform3f::look_at(position, target, up, Hand::RH);
	let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (Hand::RH, Clip::NO));
	let view_proj = projection * view;
	let inv_view_proj = view_proj.inverse();

	let cam = Camera {
		viewport,
		aspect_ratio,
		position,
		near,
		far,
		view,
		projection,
		view_proj,
		inv_view_proj,
		clip: Clip::NO,
	};

	println!("View: {:?}", cam.view);
	println!("Projection: {:?}", cam.projection);
	println!("View-Projection: {:?}", cam.view_proj);
	println!("Inverse View-Projection: {:?}", cam.inv_view_proj);

	// --- Test points ---
	let test_points = [
		Vec3::ZERO,                // Center
		Vec3(0.0, 1.0, 0.0),       // Up
		Vec3(0.0, -1.0, 0.0),      // Down
		Vec3(0.0, 0.0, 1.0),       // Forward-Z
		Vec3(0.0, 0.0, -1.0),      // Back-Z
	];

	fn print_ray_stuff(ray: &Ray3<f32>, position: Vec3f) {
		println!("  Unprojected ray: origin = {:?}, dir = {:?}", ray.origin, ray.direction);
		// Additional sanity checks:
		let dist_near_to_cam = (ray.origin - position).len();
		println!("  Distance camera to ray origin (near): {}", dist_near_to_cam);
		// Additional sanity check (dot of direction to near, with direction to origin):
		let direction_to_origin = (ray.origin - position).norm();
		let dot_product = ray.direction.norm().dot(direction_to_origin);
		println!("  Dot product of ray direction and direction to origin: {}", dot_product);
	}

	for pt in test_points.iter() {
		let screen = cam.world_to_viewport(*pt);
		println!("World {:?} => Screen {:?}", pt, screen);
		if let Some(screen_pos) = screen {
			let ray = cam.viewport_to_ray(screen_pos.cast::<i32>());
			print_ray_stuff(&ray, cam.position);
		}
	}
}

#[test]
fn test_arcball_roundtrips_position() {
	let position = Vec3(0.0, 3.2, 1.8);
	let pivot = Vec3::ZERO;
	let ref_up = Vec3::Z;

	let cam = ArcballCamera::new(position, pivot, ref_up);
	let got = cam.position();
	let diff = (got - position).len();
	assert!(diff < 1.0e-4, "ArcballCamera::new should preserve position: got={got:?} expected={position:?} diff={diff}");
}

#[test]
fn test_arcball_pan_fixed_stays_on_plane_looking_straight_down() {
	let mut cam = ArcballCamera::new(Vec3(0.0, 3.2, 1.8), Vec3::ZERO, Vec3::Z);
	cam.pitch = Angle::deg(90.0);

	let before = cam.pivot;
	cam.pan_ref(0.0, 10.0, Vec3::Z);
	let delta = cam.pivot - before;

	assert!(delta.dot(Vec3::Z).abs() < 1.0e-6, "pan_fixed should stay on the reference plane: delta={delta:?}");
	assert!(delta.len() > 0.0, "pan_fixed should keep moving when the view is straight down: delta={delta:?}");
}

#[test]
fn test_arcball_pan_fixed_stays_on_plane_looking_level() {
	let mut cam = ArcballCamera::new(Vec3(0.0, 3.2, 1.8), Vec3::ZERO, Vec3::Z);
	cam.pitch = Angle::deg(0.0);

	let before = cam.pivot;
	cam.pan_ref(10.0, 0.0, Vec3::Z);
	let sideways = cam.pivot - before;

	assert!(sideways.dot(Vec3::Z).abs() < 1.0e-6, "pan_fixed should stay on the reference plane: delta={sideways:?}");
	assert!(sideways.len() > 0.0, "pan_fixed should still move sideways when the camera is level with the plane: delta={sideways:?}");

	let before = cam.pivot;
	cam.pan_ref(0.0, 10.0, Vec3::Z);
	let forward = cam.pivot - before;

	assert!(forward.dot(Vec3::Z).abs() < 1.0e-6, "pan_fixed should stay on the reference plane: delta={forward:?}");
	assert!(forward.len() == 0.0, "pan_fixed should lose forward/back motion when the camera is level with the plane: delta={forward:?}");
}
