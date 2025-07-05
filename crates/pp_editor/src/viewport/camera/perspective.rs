use cgmath::{InnerSpace, MetricSpace};
use serde::{Deserialize, Serialize};

use crate::measures::Dimensions;

use super::Camera;

/// A perspective camera, where objects further from the eye of the camera
/// appear smaller. This camera is configured to orbit around a specific point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerspectiveCamera {
    /// The actual location of the camera
    pub eye: cgmath::Point3<f32>,
    /// Where the camera is looking at
    pub target: cgmath::Point3<f32>,
    /// The field of view of the camera
    pub fovy: f32,
    /// The near plane of the camera
    pub znear: f32,
    /// The far plane of the camera
    pub zfar: f32,
    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            eye: (4.0, 4.0, 4.0).into(),
            target: (0.0, 0.0, 0.5).into(),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            is_dirty: true,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn view_proj(&self, dims: Dimensions<f32>) -> cgmath::Matrix4<f32> {
        let aspect = dims.width.max(1.0) / dims.height.max(1.0);
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, cgmath::Vector3::unit_z());
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), aspect, self.znear, self.zfar);
        proj * view
    }

    fn eye(&self) -> [f32; 4] {
        [self.eye.x, self.eye.y, self.eye.z, 1.0]
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty
    }
}

const PERSP_SPEED_DOLLY: f32 = 0.05;
const PERSP_SPEED_ORBIT: f32 = 0.005;
const PERSP_SPEED_PAN: f32 = 0.005;
const PERSP_MAX_DISTANCE: f32 = 12.0;

impl PerspectiveCamera {
    pub fn orbit(&mut self, delta: &cgmath::Point2<f32>) {
        let forward = self.eye - self.target;
        let distance = forward.magnitude();
        // Convert to spherical coordinates
        let mut theta = forward.y.atan2(forward.x); // Azimuth angle (rotation around the vertical axis)
        let mut phi = forward.z.atan2((forward.x * forward.x + forward.y * forward.y).sqrt()); // Elevation angle
                                                                                               // Adjust angles based on input deltas
        theta -= delta.x * PERSP_SPEED_ORBIT; // Horizontal rotation
        phi += delta.y * PERSP_SPEED_ORBIT; // Vertical rotation

        // Clamp phi to avoid flipping the camera
        let epsilon = 0.01; // To avoid gimbal lock
        phi = phi
            .clamp(-std::f32::consts::FRAC_PI_2 + epsilon, std::f32::consts::FRAC_PI_2 - epsilon);

        // Convert back to Cartesian coordinates
        let new_forward = cgmath::Vector3::new(
            distance * phi.cos() * theta.cos(), // X
            distance * phi.cos() * theta.sin(), // Y
            distance * phi.sin(),               // Z
        );

        self.eye = self.target + new_forward;
        self.is_dirty = true
    }

    pub fn pan(&mut self, delta: &cgmath::Point2<f32>) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(cgmath::Vector3::unit_z()).normalize();
        let up = right.cross(forward).normalize();
        let pan_delta = right * (delta.x * PERSP_SPEED_PAN) + up * (-delta.y * PERSP_SPEED_PAN);
        self.eye -= pan_delta;
        self.target -= pan_delta;
        self.is_dirty = true;
    }

    pub fn dolly(&mut self, delta: f32) {
        let forward = self.target - self.eye;
        let new_eye = self.eye + forward * delta * PERSP_SPEED_DOLLY;
        // Ensure the new eye position does not exceed max_distance from the target
        if new_eye.distance(self.target) <= PERSP_MAX_DISTANCE {
            self.eye = new_eye;
        } else {
            self.eye = self.target - forward.normalize() * PERSP_MAX_DISTANCE;
        }

        // Mark the camera as dirty for recalculations
        self.is_dirty = true;
    }
}
