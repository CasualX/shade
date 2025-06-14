use cvmath::{Vec3, Vec4, Mat4, Rad};

// Constants
pub const CAMERA_MIN_DISTANCE: f32 = 1.0;
pub const CAMERA_MAX_DISTANCE: f32 = 100.0;
pub const CAMERA_RESET_SIGNAL: f32 = -9999.0;

/// Arcball camera with rotation, zoom, and pan support
pub struct ArcballCamera {
    pub target: Vec3<f32>,
    pub up: Vec3<f32>,
    pub distance: f32,
    rotation_x: f32,
    rotation_y: f32,
    // Store original state for reset
    original_target: Vec3<f32>,
    original_distance: f32,
}

impl ArcballCamera {
    /// Create a new arcball camera
    pub fn new(position: Vec3<f32>, target: Vec3<f32>) -> Self {
        let delta = position - target;
        let distance = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt();
        Self {
            target,
            up: Vec3::Y,
            distance,
            rotation_x: 0.0,
            rotation_y: 0.0,
            original_target: target,
            original_distance: distance,
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
            self.distance = self.distance.clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
        }
    }
    
    /// Pan the camera by moving the target
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        // Calculate camera orientation without translation
        let rot_y = Mat4::rotate(Rad(self.rotation_y), Vec3::Y);
        let rot_x = Mat4::rotate(Rad(self.rotation_x), Vec3::X);
        let rotation = rot_y * rot_x;
        
        // Extract right and up vectors from rotation matrix
        let right = Vec3::new(rotation.x().x, rotation.x().y, rotation.x().z);
        let up = Vec3::new(rotation.y().x, rotation.y().y, rotation.y().z);
        
        // Scale movement based on distance for consistent feel
        let scale = self.distance * 0.001;
        
        // Update target position
        self.target = self.target + right * (delta_x * scale) + up * (delta_y * scale);
    }
    
    /// Get the view matrix for rendering
    pub fn get_view_matrix(&self) -> Mat4<f32> {
        // Calculate camera position from rotations and distance
        let rot_y = Mat4::rotate(Rad(self.rotation_y), Vec3::Y);
        let rot_x = Mat4::rotate(Rad(self.rotation_x), Vec3::X);
        let rotation = rot_y * rot_x;
        
        let offset = rotation * Vec4::new(0.0, 0.0, self.distance, 1.0);
        let position = self.target + Vec3::new(offset.x, offset.y, offset.z);
        
        Mat4::look_at(position, self.target, self.up, cvmath::RH)
    }
    
    /// Get camera position (computed, not stored)
    pub fn get_position(&self) -> Vec3<f32> {
        let rot_y = Mat4::rotate(Rad(self.rotation_y), Vec3::Y);
        let rot_x = Mat4::rotate(Rad(self.rotation_x), Vec3::X);
        let rotation = rot_y * rot_x;
        let offset = rotation * Vec4::new(0.0, 0.0, self.distance, 1.0);
        self.target + Vec3::new(offset.x, offset.y, offset.z)
    }
    
    /// Reset camera to initial state
    pub fn reset(&mut self) {
        self.rotation_x = 0.0;
        self.rotation_y = 0.0;
        self.target = self.original_target;
        self.distance = self.original_distance;
    }
    
    /// Set camera distance
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
    }
    
    /// Get current rotation angles (x, y) in radians
    pub fn get_rotation(&self) -> (f32, f32) {
        (self.rotation_x, self.rotation_y)
    }
    
    /// Get current rotation angles in degrees
    pub fn get_rotation_degrees(&self) -> (f32, f32) {
        (self.rotation_x.to_degrees(), self.rotation_y.to_degrees())
    }
    
    /// Get camera target
    pub fn get_target(&self) -> Vec3<f32> {
        self.target
    }
    
    /// Get camera distance
    pub fn get_distance(&self) -> f32 {
        self.distance
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
        assert_eq!(camera.original_target, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(camera.original_distance, 10.0);
    }
    
    #[test]
    fn test_camera_reset() {
        let mut camera = ArcballCamera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0)
        );
        camera.update_from_input(1.0, 1.0, 0.0);
        camera.pan(10.0, 5.0);
        camera.distance = 20.0;
        camera.reset();
        assert_eq!(camera.rotation_x, 0.0);
        assert_eq!(camera.rotation_y, 0.0);
        assert_eq!(camera.target, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(camera.distance, 10.0);
    }
}