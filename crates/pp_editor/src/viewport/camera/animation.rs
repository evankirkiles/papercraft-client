//! Time-based camera moves, used to glide a camera to a framed destination
//! instead of teleporting it there.
//!
//! An animation owns the *destination*; the camera keeps its own live pose and
//! is only written to as the animation advances. Nothing here reads the clock:
//! callers hand in a frame delta, which comes from `requestAnimationFrame` by
//! way of `App::update`.

use cgmath::{InnerSpace, One, Point2, Point3, Quaternion, Rotation, Vector3};

/// How long a framing move takes, in milliseconds.
pub const FRAME_DURATION_MS: f32 = 500.0;

/// Cubic ease-out: quick departure, soft landing.
fn ease_out_cubic(s: f32) -> f32 {
    1.0 - (1.0 - s).powi(3)
}

/// The shared clock of an in-flight camera animation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Tween {
    elapsed_ms: f32,
}

impl Tween {
    /// Advances the clock by a frame delta, returning the eased 0-1 progress.
    fn advance(&mut self, dt_ms: f32) -> f32 {
        self.elapsed_ms = (self.elapsed_ms + dt_ms).min(FRAME_DURATION_MS);
        ease_out_cubic(self.elapsed_ms / FRAME_DURATION_MS)
    }

    fn is_done(&self) -> bool {
        self.elapsed_ms >= FRAME_DURATION_MS
    }
}

/// An orbit-style move of a perspective camera: the target slides in a straight
/// line while the eye swings around it along an arc, so the camera sweeps
/// around the model rather than cutting through it.
#[derive(Debug, Clone, Copy)]
pub struct OrbitAnimation {
    from_target: Point3<f32>,
    to_target: Point3<f32>,
    /// The unit direction from the target towards the eye, at the start.
    from_dir: Vector3<f32>,
    /// Rotation carrying `from_dir` onto the destination direction.
    rotation: Quaternion<f32>,
    from_distance: f32,
    to_distance: f32,
    tween: Tween,
}

impl OrbitAnimation {
    pub fn new(
        (from_eye, from_target): (Point3<f32>, Point3<f32>),
        (to_eye, to_target): (Point3<f32>, Point3<f32>),
    ) -> Self {
        let from_offset = from_eye - from_target;
        let to_offset = to_eye - to_target;
        let from_distance = from_offset.magnitude();
        let to_distance = to_offset.magnitude();
        // A degenerate offset has no direction to swing from; the vertical axis
        // is as good a starting point as any, and the distance lerp still runs.
        let from_dir =
            if from_distance > 1e-6 { from_offset / from_distance } else { Vector3::unit_z() };
        let to_dir = if to_distance > 1e-6 { to_offset / to_distance } else { Vector3::unit_z() };
        Self {
            from_target,
            to_target,
            from_dir,
            // `from_arc` handles the antiparallel case (a 180 degree flip, i.e.
            // framing something behind the camera) using the fallback axis,
            // where normalizing a lerp of the two directions would divide by
            // zero. Swinging around Z, the world's up axis, also reads more
            // naturally than an arbitrary perpendicular.
            rotation: Quaternion::from_arc(from_dir, to_dir, Some(Vector3::unit_z())),
            from_distance,
            to_distance,
            tween: Tween::default(),
        }
    }

    /// Advances by a frame delta, returning the `(eye, target)` to apply.
    pub fn advance(&mut self, dt_ms: f32) -> (Point3<f32>, Point3<f32>) {
        let s = self.tween.advance(dt_ms);
        let target = self.from_target + (self.to_target - self.from_target) * s;
        let dir = Quaternion::one().slerp(self.rotation, s).rotate_vector(self.from_dir);
        let distance = self.from_distance + (self.to_distance - self.from_distance) * s;
        (target + dir * distance, target)
    }

    pub fn is_done(&self) -> bool {
        self.tween.is_done()
    }
}

/// A pan + zoom of an orthographic camera.
#[derive(Debug, Clone, Copy)]
pub struct PanZoomAnimation {
    from_eye: Point2<f32>,
    to_eye: Point2<f32>,
    from_zoom: f32,
    to_zoom: f32,
    tween: Tween,
}

impl PanZoomAnimation {
    pub fn new(
        (from_eye, from_zoom): (Point2<f32>, f32),
        (to_eye, to_zoom): (Point2<f32>, f32),
    ) -> Self {
        Self { from_eye, to_eye, from_zoom, to_zoom, tween: Tween::default() }
    }

    /// Advances by a frame delta, returning the `(eye, zoom)` to apply.
    pub fn advance(&mut self, dt_ms: f32) -> (Point2<f32>, f32) {
        let s = self.tween.advance(dt_ms);
        let eye = Point2::new(
            self.from_eye.x + (self.to_eye.x - self.from_eye.x) * s,
            self.from_eye.y + (self.to_eye.y - self.from_eye.y) * s,
        );
        // Zoom interpolates geometrically, so each frame scales the view by the
        // same *factor* rather than the same amount - a linear lerp from 0.1 to
        // 10.0 would spend almost the whole second visually parked at the far end.
        let zoom = self.from_zoom * (self.to_zoom / self.from_zoom).powf(s);
        (eye, zoom)
    }

    pub fn is_done(&self) -> bool {
        self.tween.is_done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_hits_both_endpoints() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
    }

    #[test]
    fn tween_clamps_past_the_duration() {
        let mut tween = Tween::default();
        assert_eq!(tween.advance(FRAME_DURATION_MS * 10.0), 1.0);
        assert!(tween.is_done());
    }

    #[test]
    fn orbit_lands_exactly_on_the_destination() {
        let from = (Point3::new(4.0, 4.0, 4.0), Point3::new(0.0, 0.0, 0.0));
        let to = (Point3::new(-2.0, 1.0, 3.0), Point3::new(1.0, 1.0, 1.0));
        let mut anim = OrbitAnimation::new(from, to);
        let (eye, target) = anim.advance(FRAME_DURATION_MS);
        assert!(anim.is_done());
        assert!((eye - to.0).magnitude() < 1e-3, "eye {eye:?} should reach {:?}", to.0);
        assert!((target - to.1).magnitude() < 1e-5);
    }

    #[test]
    fn a_180_degree_flip_stays_finite() {
        // Directly antiparallel directions: normalizing a lerp of these would
        // divide by zero halfway through.
        let origin = Point3::new(0.0, 0.0, 0.0);
        let from = (Point3::new(0.0, 5.0, 0.0), origin);
        let to = (Point3::new(0.0, -5.0, 0.0), origin);
        let mut anim = OrbitAnimation::new(from, to);
        for _ in 0..63 {
            let (eye, target) = anim.advance(16.0);
            assert!(eye.x.is_finite() && eye.y.is_finite() && eye.z.is_finite(), "eye {eye:?}");
            let distance = (eye - target).magnitude();
            assert!((distance - 5.0).abs() < 1e-3, "distance {distance} should stay at 5");
        }
    }

    #[test]
    fn zoom_interpolates_monotonically_and_exactly() {
        let mut anim =
            PanZoomAnimation::new((Point2::new(0.0, 0.0), 0.1), (Point2::new(0.0, 0.0), 4.0));
        let mut last = 0.1;
        for _ in 0..62 {
            let (_, zoom) = anim.advance(16.0);
            assert!(zoom >= last, "zoom {zoom} should not go backwards from {last}");
            last = zoom;
        }
        let (eye, zoom) = anim.advance(FRAME_DURATION_MS);
        assert!(anim.is_done());
        assert!((zoom - 4.0).abs() < 1e-4, "zoom {zoom} should land on 4.0");
        assert_eq!(eye, Point2::new(0.0, 0.0));
    }
}
