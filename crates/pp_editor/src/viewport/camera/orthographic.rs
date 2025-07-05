use serde::{Deserialize, Serialize};

use crate::measures::Dimensions;

use super::Camera;

/// An orthographic camera, where objects are the same size regardless of their
/// distance from the camera.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrthographicCamera {
    /// The position of the camera
    pub eye: cgmath::Point2<f32>,
    /// The distance of the camera from the Z plane
    pub zoom: f32,
    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self { eye: (0.0, 0.0).into(), zoom: 0.5, is_dirty: true }
    }
}

const ORTHO_SPEED_DOLLY: f32 = 0.03;
const ORTHO_SPEED_PAN: f32 = 0.003;
const ORTHO_MAX_ZOOM: f32 = 10.0;
const ORTHO_MIN_ZOOM: f32 = 0.1;

impl Camera for OrthographicCamera {
    fn view_proj(&self, delta: Dimensions<f32>) -> cgmath::Matrix4<f32> {
        let aspect = delta.width.max(1.0) / delta.height.max(1.0);
        let half_width = aspect / self.zoom;
        let half_height = 1.0 / self.zoom;
        let view = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            -1.0 * self.eye.x,
            -1.0 * self.eye.y,
            -1.0,
        ));
        let proj = cgmath::ortho(-half_width, half_width, -half_height, half_height, -1.1, 1.1);
        proj * view
    }

    fn eye(&self) -> [f32; 4] {
        [self.eye.x, self.eye.y, 1.0, 0.0]
    }
}

impl OrthographicCamera {
    pub fn pan(&mut self, delta: &cgmath::Point2<f32>) {
        self.eye.x -= delta.x * ORTHO_SPEED_PAN / self.zoom;
        self.eye.y += delta.y * ORTHO_SPEED_PAN / self.zoom;
        self.is_dirty = true;
    }

    pub fn zoom(&mut self, delta: f32) {
        let new_zoom = self.zoom * (1.0 + delta * ORTHO_SPEED_DOLLY);
        self.zoom = new_zoom.clamp(ORTHO_MIN_ZOOM, ORTHO_MAX_ZOOM);
        self.is_dirty = true;
    }
}
