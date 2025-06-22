use super::*;

#[test]
fn test_camera_setup() {
	let viewport = Bounds2::new(Vec2i(0, 0), Vec2i(100, 100));
	let aspect_ratio = 1.0;
	let fov_y = Rad::quarter();
	let near = 1.0;
	let far = 10.0;

	let position = Vec3::new(5.0, 0.0, 0.0);
	let target = Vec3::ZERO;
	let up = Vec3::Y;

	println!("Viewport: {:?}", viewport);
	println!("Aspect ratio: {}", aspect_ratio);
	println!("FOV (Y): {:?}", fov_y);
	println!("Near: {}", near);
	println!("Far: {}", far);
	println!("Camera position: {:?}", position);
	println!("Camera target: {:?}", target);
	println!("Camera forward = {:?}", (target - position).normalize());
	println!("Camera up: {:?}", up);

	let view = Mat4::look_at(position, target, up, Hand::RH);
	let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (Hand::RH, Clip::NO));
	let view_proj = projection * view;
	let inv_view_proj = view_proj.inverse();

	let cam = CameraSetup {
		viewport,
		aspect_ratio,
		position,
		near,
		far,
		view,
		projection,
		view_proj,
		inv_view_proj,
		clip: Clip::NO, // Assuming NDC z = -1..1
	};

	println!("View: {:?}", cam.view);
	println!("Projection: {:?}", cam.projection);
	println!("View-Projection: {:?}", cam.view_proj);
	println!("Inverse View-Projection: {:?}", cam.inv_view_proj);

	// --- Test points ---
	let test_points = [
		Vec3::ZERO,                     // Center
		Vec3::new(0.0, 1.0, 0.0),       // Up
		Vec3::new(0.0, -1.0, 0.0),      // Down
		Vec3::new(0.0, 0.0, 1.0),       // Forward-Z
		Vec3::new(0.0, 0.0, -1.0),      // Back-Z
	];

	fn print_ray_stuff(ray: &Ray<f32>, position: Vec3f) {
		println!("  Unprojected ray: origin = {:?}, dir = {:?}", ray.origin, ray.direction);
		// Additional sanity checks:
		let dist_near_to_cam = (ray.origin - position).len();
		println!("  Distance camera to ray origin (near): {}", dist_near_to_cam);
		// Additional sanity check (dot of direction to near, with direction to origin):
		let direction_to_origin = (ray.origin - position).normalize();
		let dot_product = ray.direction.normalize().dot(direction_to_origin);
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
