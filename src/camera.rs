use cvmath::{Vec3, Vec4, Mat4, Rad};

/// Simple rotation state for camera control
/// Using Euler angles for simplicity, can be upgraded to quaternions later
pub struct ArcballCamera {
    pub position: Vec3<f32>,
    pub target: Vec3<f32>,
    pub up: Vec3<f32>,
    pub distance: f32,
    rotation_x: f32,
    rotation_y: f32,
}

impl ArcballCamera {
    /// Create a new arcball camera
    pub fn new(position: Vec3<f32>, target: Vec3<f32>) -> Self {
        let delta = position - target;
        let distance = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt();
        Self {
            position,
            target,
            up: Vec3::Y,
            distance,
            rotation_x: 0.0,
            rotation_y: 0.0,
        }
    }
    
    /// Update camera from input deltas
    pub fn update_from_input(&mut self, delta_x: f32, delta_y: f32, delta_zoom: f32) {
        // Update rotation angles
        if delta_x != 0.0 || delta_y != 0.0 {
            self.rotation_y += delta_x * 0.01;
            self.rotation_x += delta_y * 0.01;
            
            // Clamp X rotation to prevent flipping
            self.rotation_x = self.rotation_x.clamp(-1.5, 1.5);
        }
        
        // Handle zoom
        if delta_zoom != 0.0 {
            self.distance *= 1.0 + delta_zoom * 0.001;
            self.distance = self.distance.clamp(1.0, 100.0);
        }
    }
    
    /// Get the view matrix for rendering
    pub fn get_view_matrix(&self) -> Mat4<f32> {
        // Calculate camera position from rotations and distance
        let rot_y = Mat4::rotate(Rad(self.rotation_y), Vec3::Y);
        let rot_x = Mat4::rotate(Rad(self.rotation_x), Vec3::X);
        let rotation = rot_y * rot_x;
        
        let offset = rotation * Vec4::new(0.0, 0.0, self.distance, 1.0);
        let position = self.target + offset.xyz();
        
        Mat4::look_at(position, self.target, self.up, cvmath::RH)
    }
    
    /// Reset camera to initial state
    pub fn reset(&mut self) {
        self.rotation_x = 0.0;
        self.rotation_y = 0.0;
        let delta = self.position - self.target;
        self.distance = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt();
    }
    
    /// Set camera distance
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(1.0, 100.0);
    }
    
    /// Get current rotation angles (x, y)
    pub fn get_rotation(&self) -> (f32, f32) {
        (self.rotation_x, self.rotation_y)
    }
    
    /// Set rotation angles
    pub fn set_rotation(&mut self, rotation_x: f32, rotation_y: f32) {
        self.rotation_x = rotation_x.clamp(-1.5, 1.5);
        self.rotation_y = rotation_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_creation() {
        let camera = ArcballCamera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0)
        );
        assert_eq!(camera.distance, 10.0);
        assert_eq!(camera.target, Vec3::new(0.0, 0.0, 0.0));
    }
    
    #[test]
    fn test_camera_reset() {
        let mut camera = ArcballCamera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0)
        );
        camera.update_from_input(1.0, 1.0, 0.0);
        camera.reset();
        assert_eq!(camera.rotation_x, 0.0);
        assert_eq!(camera.rotation_y, 0.0);
    }
}