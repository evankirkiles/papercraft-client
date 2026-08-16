use cgmath::{InnerSpace, MetricSpace};
use serde::{Deserialize, Serialize};

use pp_core::measures::Dimensions;

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
    /// The far plane of the camera. This is a floor; the effective far plane
    /// used for rendering also expands to keep the whole model in view (see
    /// [`Self::effective_zfar`]).
    pub zfar: f32,
    /// The radius of the document's bounding sphere (world units), refreshed
    /// once per frame by the renderer. Used to keep the far plane and the
    /// dolly-out limit ahead of the model's actual size.
    #[serde(skip)]
    pub fit_radius: f32,
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
            fit_radius: 0.0,
            is_dirty: true,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn view_proj(&self, dims: Dimensions<f32>) -> cgmath::Matrix4<f32> {
        let aspect = dims.width.max(1.0) / dims.height.max(1.0);
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, cgmath::Vector3::unit_z());
        let proj =
            cgmath::perspective(cgmath::Deg(self.fovy), aspect, self.znear, self.effective_zfar());
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
/// Extra breathing room around the model's bounding sphere when computing
/// how far away dollying out is allowed to go.
const PERSP_FIT_MARGIN: f32 = 1.3;

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

    /// Pans the camera. Speed scales with the camera's distance from its
    /// orbit target, so a given screen-space drag covers the same apparent
    /// fraction of the view whether zoomed in close or dollied far out.
    pub fn pan(&mut self, delta: &cgmath::Point2<f32>) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(cgmath::Vector3::unit_z()).normalize();
        let up = right.cross(forward).normalize();
        let speed = PERSP_SPEED_PAN * self.eye.distance(self.target);
        let pan_delta = right * (delta.x * speed) + up * (-delta.y * speed);
        self.eye -= pan_delta;
        self.target -= pan_delta;
        self.is_dirty = true;
    }

    /// Applies an incremental dolly step. The max distance from the target
    /// relaxes automatically (see [`Self::max_distance`]) so the whole model
    /// always stays reachable, even if it's larger than [`PERSP_MAX_DISTANCE`]
    /// was tuned for.
    pub fn dolly(&mut self, delta: f32) {
        let forward = self.target - self.eye;
        let new_eye = self.eye + forward * delta * PERSP_SPEED_DOLLY;
        let max_distance = self.max_distance();
        // Ensure the new eye position does not exceed max_distance from the target
        if new_eye.distance(self.target) <= max_distance {
            self.eye = new_eye;
        } else {
            self.eye = self.target - forward.normalize() * max_distance;
        }

        // Mark the camera as dirty for recalculations
        self.is_dirty = true;
    }

    /// Refreshes the document's bounding-sphere radius, called once per frame
    /// by the renderer so the far plane and dolly limit stay ahead of the
    /// model's current size without requiring user interaction first.
    pub fn sync_fit_radius(&mut self, fit_radius: f32) {
        self.fit_radius = fit_radius;
    }

    fn max_distance(&self) -> f32 {
        if self.fit_radius <= 0.0 {
            return PERSP_MAX_DISTANCE;
        }
        let half_fov = self.fovy.to_radians() / 2.0;
        let needed = (self.fit_radius * PERSP_FIT_MARGIN) / half_fov.sin();
        needed.max(PERSP_MAX_DISTANCE)
    }

    /// The far plane actually used for rendering: at least `self.zfar`, but
    /// expanded to comfortably clear the model regardless of where the
    /// camera currently sits, so dollying/orbiting can never clip it.
    fn effective_zfar(&self) -> f32 {
        let needed = self.eye.distance(self.target) + self.fit_radius * PERSP_FIT_MARGIN;
        self.zfar.max(needed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_speed_scales_with_distance_from_target() {
        let mut near = PerspectiveCamera { eye: (2.0, 0.0, 0.0).into(), ..Default::default() };
        let mut far = PerspectiveCamera { eye: (20.0, 0.0, 0.0).into(), ..Default::default() };
        let delta = cgmath::Point2::new(10.0, 0.0);
        near.pan(&delta);
        far.pan(&delta);
        let near_shift = (near.eye - cgmath::Point3::new(2.0, 0.0, 0.0)).magnitude();
        let far_shift = (far.eye - cgmath::Point3::new(20.0, 0.0, 0.0)).magnitude();
        assert!(
            far_shift > near_shift,
            "far_shift {far_shift} should exceed near_shift {near_shift}"
        );
    }

    #[test]
    fn dolly_clamps_out_far_enough_to_fit_a_large_model() {
        // A 200x200x200 cm cube, as produced by scaling the dimensions panel.
        let fit_radius = (200.0_f32.powi(2) * 3.0).sqrt() / 2.0;
        let mut camera = PerspectiveCamera::default();
        camera.sync_fit_radius(fit_radius);
        for _ in 0..10_000 {
            camera.dolly(-1.0);
        }
        let distance = camera.eye.distance(camera.target);
        assert!(
            distance >= fit_radius,
            "distance {distance} should be able to fit fit_radius {fit_radius}"
        );
    }

    #[test]
    fn effective_zfar_never_clips_a_large_model() {
        let fit_radius = 500.0;
        let mut camera = PerspectiveCamera::default();
        camera.sync_fit_radius(fit_radius);
        camera.eye = camera.target - cgmath::Vector3::new(1.0, 0.0, 0.0);
        // Farthest point of the bounding sphere from the eye.
        let farthest = camera.eye.distance(camera.target) + fit_radius;
        assert!(camera.effective_zfar() >= farthest);
    }

    #[test]
    fn small_model_keeps_default_limits() {
        let camera = PerspectiveCamera::default();
        assert_eq!(camera.max_distance(), PERSP_MAX_DISTANCE);
        assert_eq!(camera.effective_zfar(), camera.zfar);
    }
}
