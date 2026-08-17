use cgmath::{EuclideanSpace, InnerSpace, MetricSpace};
use serde::{Deserialize, Serialize};

use pp_core::{bounds::Aabb3, measures::Dimensions};

use super::{animation::OrbitAnimation, Camera};

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
    /// The framing move currently in flight, if any. Transient, like
    /// [`Self::fit_radius`], so it never lands in a persisted layout.
    #[serde(skip)]
    pub animation: Option<OrbitAnimation>,
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
            animation: None,
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
/// How much of the frame a framed selection should span, so it lands centered
/// with plenty of its surroundings still in view rather than filling the view.
const PERSP_FRAME_FILL: f32 = 0.5;
/// How close framing may put the eye to the selection. Selections with no
/// extent (a single vertex) would otherwise frame to a distance of zero, which
/// puts the target behind the near plane.
const PERSP_MIN_FRAME_DISTANCE: f32 = 1.0;
/// Kept away from straight-down/straight-up, where the view direction would be
/// parallel to the hardcoded Z up-vector and `look_at_rh` would return NaN.
const PERSP_MAX_ELEVATION: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

impl PerspectiveCamera {
    /// The unit direction at azimuth `theta` and elevation `phi`, with the
    /// elevation clamped away from the poles: there the direction would be
    /// parallel to the Z up-vector used by [`Camera::view_proj`] and the view
    /// matrix would come out NaN. Every direction this camera adopts - orbited
    /// or framed - passes through here.
    fn clamped_dir(theta: f32, phi: f32) -> cgmath::Vector3<f32> {
        let phi = phi.clamp(-PERSP_MAX_ELEVATION, PERSP_MAX_ELEVATION);
        cgmath::Vector3::new(
            phi.cos() * theta.cos(), // X
            phi.cos() * theta.sin(), // Y
            phi.sin(),               // Z
        )
    }

    /// [`Self::clamped_dir`] applied to an existing direction, normalizing it
    /// along the way.
    fn clamp_elevation(dir: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        Self::clamped_dir(dir.y.atan2(dir.x), dir.z.atan2((dir.x * dir.x + dir.y * dir.y).sqrt()))
    }

    pub fn orbit(&mut self, delta: &cgmath::Point2<f32>) {
        self.cancel_animation();
        let forward = self.eye - self.target;
        let distance = forward.magnitude();
        // Convert to spherical coordinates, adjusting the angles by the input
        // deltas, then back to Cartesian
        let theta = forward.y.atan2(forward.x) - delta.x * PERSP_SPEED_ORBIT; // Horizontal rotation
        let phi = forward.z.atan2((forward.x * forward.x + forward.y * forward.y).sqrt())
            + delta.y * PERSP_SPEED_ORBIT; // Vertical rotation

        self.eye = self.target + Self::clamped_dir(theta, phi) * distance;
        self.is_dirty = true
    }

    /// Pans the camera. Speed scales with the camera's distance from its
    /// orbit target, so a given screen-space drag covers the same apparent
    /// fraction of the view whether zoomed in close or dollied far out.
    pub fn pan(&mut self, delta: &cgmath::Point2<f32>) {
        self.cancel_animation();
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
        self.cancel_animation();
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

    /// The `(eye, target)` this camera would need to center `aabb` at
    /// [`PERSP_FRAME_FILL`] of the frame, looking at it along `normal` (falling
    /// back to the direction the camera already faces from). `None` if there is
    /// nothing to frame.
    ///
    /// `aspect` is the viewport's width over its height, as used by
    /// [`Camera::view_proj`].
    pub fn frame_destination(
        &self,
        aabb: &Aabb3,
        normal: Option<cgmath::Vector3<f32>>,
        aspect: f32,
    ) -> Option<(cgmath::Point3<f32>, cgmath::Point3<f32>)> {
        if aabb.is_empty() {
            return None;
        }
        let target = cgmath::Point3::from_vec(aabb.center());
        let dir = Self::clamp_elevation(normal.unwrap_or(self.eye - self.target));

        // Basis of the destination view: the camera sits at `target + dir`, so
        // it looks back along `-dir`.
        let forward = -dir;
        let right = forward.cross(cgmath::Vector3::unit_z()).normalize();
        let up = right.cross(forward).normalize();

        // `cgmath::perspective` takes a *vertical* fov, so the horizontal one
        // follows from the aspect ratio. A portrait viewport is bound by the
        // horizontal half-angle, a landscape one by the vertical.
        let tan_half_y = (self.fovy.to_radians() / 2.0).tan();
        let tan_half_x = tan_half_y * aspect;

        // Rather than fitting the box's bounding sphere - which is very loose
        // for the flat faces and pieces selected here - project each corner
        // into the destination view and take the distance that leaves the
        // worst one spanning `PERSP_FRAME_FILL` of the frustum.
        let distance = aabb
            .corners()
            .into_iter()
            .map(|corner| {
                let offset = cgmath::Point3::from_vec(corner) - target;
                let (x, y, depth) = (offset.dot(right), offset.dot(up), offset.dot(forward));
                // At an eye distance D the corner sits `depth + D` in front of
                // the camera, and spans `|x| / (tan_x * (depth + D))` of the
                // frame's half-width. Corners nearer the camera (negative
                // depth) are the binding ones, hence the subtraction.
                let half_extent =
                    (x.abs() / tan_half_x).max(y.abs() / tan_half_y) / PERSP_FRAME_FILL;
                half_extent - depth
            })
            .fold(f32::NEG_INFINITY, f32::max);
        let distance = distance.max(PERSP_MIN_FRAME_DISTANCE);

        Some((target + dir * distance, target))
    }

    /// Starts a framing move towards [`Self::frame_destination`], replacing any
    /// move already in flight.
    pub fn animate_to_frame(
        &mut self,
        aabb: &Aabb3,
        normal: Option<cgmath::Vector3<f32>>,
        aspect: f32,
    ) {
        let Some(destination) = self.frame_destination(aabb, normal, aspect) else { return };
        self.animation = Some(OrbitAnimation::new((self.eye, self.target), destination));
    }

    /// Advances an in-flight framing move by a frame delta.
    pub fn tick(&mut self, dt_ms: f32) {
        let Some(animation) = self.animation.as_mut() else { return };
        (self.eye, self.target) = animation.advance(dt_ms);
        if animation.is_done() {
            self.animation = None;
        }
        self.is_dirty = true;
    }

    /// Abandons any in-flight framing move, leaving the camera where it is.
    /// Every camera input calls this, so touching the camera always wins over
    /// an in-flight framing move.
    pub fn cancel_animation(&mut self) {
        self.animation = None;
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
    use super::{super::animation::FRAME_DURATION_MS, *};

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

    fn unit_cube() -> Aabb3 {
        Aabb3 {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Every corner of `aabb` projects inside the frame, and the widest of them
    /// lands near `PERSP_FRAME_FILL` of it - so the selection is neither
    /// clipped nor lost in the middle of the view.
    fn assert_frames_to_fill(camera: &PerspectiveCamera, aabb: &Aabb3, aspect: f32) {
        use cgmath::{EuclideanSpace, Transform};
        let dims = Dimensions { width: aspect * 100.0, height: 100.0 };
        let view_proj = camera.view_proj(dims);
        let widest = aabb
            .corners()
            .into_iter()
            .map(|corner| {
                let ndc = view_proj.transform_point(cgmath::Point3::from_vec(corner));
                // Clip space is only meaningful in front of the camera
                assert!(ndc.z > 0.0, "corner {corner:?} should be in front of the camera");
                assert!(
                    ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0,
                    "corner {corner:?} projects to {ndc:?}, outside the frame"
                );
                ndc.x.abs().max(ndc.y.abs())
            })
            .fold(0.0_f32, f32::max);
        // The binding corner sits at the fill fraction; perspective foreshortening
        // pulls the others in, so allow a little slack below it.
        assert!(
            (0.8 * PERSP_FRAME_FILL..=PERSP_FRAME_FILL + 0.02).contains(&widest),
            "widest corner reached {widest} of the frame, expected about {PERSP_FRAME_FILL}"
        );
    }

    #[test]
    fn frames_a_cube_from_its_normal() {
        let aabb = unit_cube();
        let normal = cgmath::Vector3::new(1.0, 0.0, 0.0);
        let mut camera = PerspectiveCamera::default();
        let (eye, target) = camera.frame_destination(&aabb, Some(normal), 1.0).unwrap();
        assert_eq!(target, cgmath::Point3::new(0.0, 0.0, 0.0));
        // Looking straight down the normal: the eye is on the +X axis
        assert!(eye.x > 0.0 && eye.y.abs() < 1e-5 && eye.z.abs() < 1e-5, "eye {eye:?}");
        camera.eye = eye;
        camera.target = target;
        assert_frames_to_fill(&camera, &aabb, 1.0);
    }

    #[test]
    fn portrait_viewport_frames_farther_than_landscape() {
        let aabb = unit_cube();
        let camera = PerspectiveCamera::default();
        let normal = Some(cgmath::Vector3::new(1.0, 0.0, 0.0));
        let (landscape, target) = camera.frame_destination(&aabb, normal, 2.0).unwrap();
        let (portrait, _) = camera.frame_destination(&aabb, normal, 0.5).unwrap();
        assert!(
            portrait.distance(target) > landscape.distance(target),
            "portrait {portrait:?} should sit farther back than landscape {landscape:?}"
        );
    }

    #[test]
    fn frames_a_flat_selection_consistently_in_both_orientations() {
        // A flat, wide quad facing +X - the shape most selections have here.
        let aabb = Aabb3 {
            min: cgmath::Vector3::new(0.0, -3.0, -0.5),
            max: cgmath::Vector3::new(0.0, 3.0, 0.5),
        };
        let normal = Some(cgmath::Vector3::new(1.0, 0.0, 0.0));
        for aspect in [2.0, 0.5] {
            let mut camera = PerspectiveCamera::default();
            let (eye, target) = camera.frame_destination(&aabb, normal, aspect).unwrap();
            camera.eye = eye;
            camera.target = target;
            assert_frames_to_fill(&camera, &aabb, aspect);
        }
    }

    /// A Z-facing normal - any horizontal face - is parallel to the up-vector
    /// `view_proj` hardcodes, which would make `look_at_rh` return NaN.
    #[test]
    fn framing_a_z_facing_normal_yields_a_finite_view_proj() {
        let aabb = unit_cube();
        let mut camera = PerspectiveCamera::default();
        for normal in [cgmath::Vector3::unit_z(), -cgmath::Vector3::unit_z()] {
            let (eye, target) = camera.frame_destination(&aabb, Some(normal), 1.6).unwrap();
            camera.eye = eye;
            camera.target = target;
            let view_proj = camera.view_proj(Dimensions { width: 160.0, height: 100.0 });
            let m: [[f32; 4]; 4] = view_proj.into();
            assert!(
                m.iter().flatten().all(|v| v.is_finite()),
                "view_proj went non-finite for normal {normal:?}: {m:?}"
            );
        }
    }

    #[test]
    fn a_180_degree_flip_stays_finite() {
        let mut camera = PerspectiveCamera {
            eye: (0.0, 5.0, 0.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            ..Default::default()
        };
        camera.animate_to_frame(&unit_cube(), Some(cgmath::Vector3::new(0.0, -1.0, 0.0)), 1.0);
        for _ in 0..70 {
            camera.tick(16.0);
            let eye = camera.eye;
            assert!(eye.x.is_finite() && eye.y.is_finite() && eye.z.is_finite(), "eye {eye:?}");
            assert!(eye.distance(camera.target) > 0.1, "camera fell into its own target");
        }
        assert!(
            camera.eye.y < 0.0,
            "camera should have swung to the -Y side, got {:?}",
            camera.eye
        );
    }

    #[test]
    fn animation_lands_on_the_destination_and_clears() {
        let aabb = unit_cube();
        let normal = Some(cgmath::Vector3::new(1.0, 0.0, 0.0));
        let mut camera = PerspectiveCamera::default();
        let (eye, target) = camera.frame_destination(&aabb, normal, 1.0).unwrap();
        camera.animate_to_frame(&aabb, normal, 1.0);
        // The full duration at 60fps, plus a frame of overshoot
        for _ in 0..(FRAME_DURATION_MS / 16.0) as usize + 1 {
            camera.tick(16.0);
        }
        assert!(camera.animation.is_none(), "animation should have cleared");
        assert!(camera.eye.distance(eye) < 1e-3, "eye {:?} should reach {eye:?}", camera.eye);
        assert!(camera.target.distance(target) < 1e-4);
    }

    #[test]
    fn camera_input_cancels_the_animation() {
        let aabb = unit_cube();
        let normal = Some(cgmath::Vector3::new(1.0, 0.0, 0.0));
        let delta = cgmath::Point2::new(4.0, 4.0);
        let inputs: [(&str, fn(&mut PerspectiveCamera, &cgmath::Point2<f32>)); 3] = [
            ("orbit", |c, d| c.orbit(d)),
            ("pan", |c, d| c.pan(d)),
            ("dolly", |c, d| c.dolly(d.y)),
        ];
        for (name, input) in inputs {
            let mut camera = PerspectiveCamera::default();
            camera.animate_to_frame(&aabb, normal, 1.0);
            camera.tick(16.0);
            assert!(camera.animation.is_some());
            input(&mut camera, &delta);
            assert!(camera.animation.is_none(), "{name} should have cancelled the animation");
        }
    }

    #[test]
    fn a_single_vertex_selection_stays_outside_the_near_plane() {
        // A selection with no extent at all
        let point = cgmath::Vector3::new(2.0, 2.0, 2.0);
        let aabb = Aabb3 { min: point, max: point };
        let camera = PerspectiveCamera::default();
        let (eye, target) = camera.frame_destination(&aabb, None, 1.0).unwrap();
        assert!(
            eye.distance(target) > camera.znear,
            "eye landed {} from the target, inside znear {}",
            eye.distance(target),
            camera.znear
        );
    }

    #[test]
    fn an_empty_selection_has_nothing_to_frame() {
        let camera = PerspectiveCamera::default();
        assert!(camera.frame_destination(&Aabb3::EMPTY, None, 1.0).is_none());
    }
}
