use serde::{Deserialize, Serialize};

use pp_core::{
    bounds::Aabb3,
    measures::{Dimensions, Rect},
};

use super::{animation::PanZoomAnimation, Camera};

/// An orthographic camera, where objects are the same size regardless of their
/// distance from the camera.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrthographicCamera {
    /// The position of the camera
    pub eye: cgmath::Point2<f32>,
    /// The distance of the camera from the Z plane
    pub zoom: f32,
    /// The framing move currently in flight, if any. Transient, so it never
    /// lands in a persisted layout.
    #[serde(skip)]
    pub animation: Option<PanZoomAnimation>,
    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self { eye: (4.0, -4.0).into(), zoom: 0.1, animation: None, is_dirty: true }
    }
}

const ORTHO_SPEED_DOLLY: f32 = 0.03;
const ORTHO_SPEED_PAN: f32 = 0.003;
const ORTHO_MAX_ZOOM: f32 = 10.0;
const ORTHO_MIN_ZOOM: f32 = 0.05;
/// Extra breathing room around the model's bounding sphere when computing
/// how far out zooming is allowed to go.
const ORTHO_FIT_MARGIN: f32 = 1.3;
/// How much farther than the nominal limit the camera may pull back, i.e. the
/// factor by which the maximum viewable area's half-extent grows.
const ORTHO_MAX_DISTANCE_SCALE: f32 = 2.0;
/// How much of the frame a framed selection should span, so it lands centered
/// with plenty of its surroundings still in view rather than filling the view.
const ORTHO_FRAME_FILL: f32 = 0.5;

impl Camera for OrthographicCamera {
    fn view_proj(&self, dims: Dimensions<f32>) -> cgmath::Matrix4<f32> {
        let aspect = dims.width.max(1.0) / dims.height.max(1.0);
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

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty
    }
}

impl OrthographicCamera {
    /// A camera that fills the frame with `rect` exactly, edge to edge.
    ///
    /// [`Camera::view_proj`] shows a half-height of `1/zoom` world units and a
    /// half-width of `aspect/zoom`, so pinning the half-height to half the
    /// rect's height frames the width to match - provided the viewport's aspect
    /// equals the rect's, which is how printing sizes its target.
    ///
    /// `rect.y` is the top edge, and the printable quadrant runs downward, so
    /// the center sits *below* it. Unlike [`Self::frame_destination`] this is
    /// exact rather than fitted: a printed page must not lose a millimeter to
    /// margin, nor pick up a neighbouring sheet's pieces.
    pub fn framing(rect: &Rect<f32>) -> Self {
        Self {
            eye: cgmath::Point2::new(rect.x + rect.width / 2.0, rect.y - rect.height / 2.0),
            zoom: 2.0 / rect.height,
            animation: None,
            is_dirty: true,
        }
    }

    pub fn pan(&mut self, delta: &cgmath::Point2<f32>) {
        self.cancel_animation();
        self.eye.x -= delta.x * ORTHO_SPEED_PAN / self.zoom;
        self.eye.y += delta.y * ORTHO_SPEED_PAN / self.zoom;
        self.is_dirty = true;
    }

    /// Applies an incremental zoom step. `fit_radius` is the radius of the
    /// model's bounding sphere (world units); the minimum zoom (i.e. how far
    /// out the camera can go) relaxes automatically so the whole model always
    /// stays reachable, even if it's larger than [`ORTHO_MIN_ZOOM`] was tuned
    /// for.
    pub fn zoom(&mut self, delta: f32, fit_radius: f32) {
        self.cancel_animation();
        let new_zoom = self.zoom * (1.0 + delta * ORTHO_SPEED_DOLLY);
        self.zoom = new_zoom.clamp(Self::min_zoom_for(fit_radius), ORTHO_MAX_ZOOM);
        self.is_dirty = true;
    }

    /// The `(eye, zoom)` that centers `aabb` at [`ORTHO_FRAME_FILL`] of the
    /// frame, or `None` if there is nothing to frame. Only the XY extent
    /// matters: the cutting viewport looks straight down at pieces lying on the
    /// Z=0 plane.
    ///
    /// `aspect` is the viewport's width over its height, as used by
    /// [`Camera::view_proj`].
    ///
    /// Note that the zoom stays inside the same limits scrolling obeys. Those
    /// are derived from the *folded* model's radius, while unfolded pieces can
    /// sprawl wider, so a very large piece selection may not fit entirely -
    /// a pre-existing quirk of the zoom-out limit rather than of framing.
    pub fn frame_destination(
        &self,
        aabb: &Aabb3,
        aspect: f32,
        fit_radius: f32,
    ) -> Option<(cgmath::Point2<f32>, f32)> {
        if aabb.is_empty() {
            return None;
        }
        let center = aabb.center();
        let size = aabb.size();
        // `view_proj` shows a half-height of 1/zoom and a half-width of
        // aspect/zoom, so the binding half-extent is whichever of the box's two
        // dimensions needs more room.
        let half_height = (size.y / 2.0).max(size.x / (2.0 * aspect)) / ORTHO_FRAME_FILL;
        let zoom = if half_height > 1e-6 { 1.0 / half_height } else { ORTHO_MAX_ZOOM };
        let zoom = zoom.clamp(Self::min_zoom_for(fit_radius), ORTHO_MAX_ZOOM);
        Some((cgmath::Point2::new(center.x, center.y), zoom))
    }

    /// Starts a framing move towards [`Self::frame_destination`], replacing any
    /// move already in flight.
    pub fn animate_to_frame(&mut self, aabb: &Aabb3, aspect: f32, fit_radius: f32) {
        let Some(destination) = self.frame_destination(aabb, aspect, fit_radius) else { return };
        self.animation = Some(PanZoomAnimation::new((self.eye, self.zoom), destination));
    }

    /// Advances an in-flight framing move by a frame delta.
    pub fn tick(&mut self, dt_ms: f32) {
        let Some(animation) = self.animation.as_mut() else { return };
        (self.eye, self.zoom) = animation.advance(dt_ms);
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

    fn min_zoom_for(fit_radius: f32) -> f32 {
        let nominal = if fit_radius <= 0.0 {
            ORTHO_MIN_ZOOM
        } else {
            (1.0 / (fit_radius * ORTHO_FIT_MARGIN)).min(ORTHO_MIN_ZOOM)
        };
        nominal / ORTHO_MAX_DISTANCE_SCALE
    }
}

#[cfg(test)]
mod tests {
    use super::{super::animation::FRAME_DURATION_MS, *};

    #[test]
    fn small_model_keeps_default_min_zoom() {
        assert_eq!(
            OrthographicCamera::min_zoom_for(0.5),
            ORTHO_MIN_ZOOM / ORTHO_MAX_DISTANCE_SCALE
        );
    }

    #[test]
    fn zoom_clamps_out_far_enough_to_fit_a_large_model() {
        // A 200x200x200 cm cube, as produced by scaling the dimensions panel.
        let fit_radius = (200.0_f32.powi(2) * 3.0).sqrt() / 2.0;
        let mut camera = OrthographicCamera { zoom: ORTHO_MAX_ZOOM, ..Default::default() };
        // Zoom out as far as the input allows.
        for _ in 0..10_000 {
            camera.zoom(-1.0, fit_radius);
        }
        let half_height = 1.0 / camera.zoom;
        assert!(
            half_height >= fit_radius,
            "half_height {half_height} should be able to fit fit_radius {fit_radius}"
        );
    }

    /// The print pass leans on this being exact: an A4 sheet's corners have to
    /// land on the corners of the image, or pieces get clipped or shrunk.
    #[test]
    fn framing_maps_a_rect_onto_the_whole_frame() {
        // The second sheet of the second row of an A4 grid, in the printable
        // quadrant that runs right and down from the origin.
        let page = Rect { x: 21.0, y: -29.7, width: 21.0, height: 29.7 };
        let camera = OrthographicCamera::framing(&page);
        // A target whose aspect matches the page's, as `PageSize::pixels` gives
        let view_proj = camera.view_proj(Dimensions { width: 2480.0, height: 3508.0 });

        let corners = [
            (page.x, page.y, -1.0, 1.0),                            // top-left
            (page.x + page.width, page.y, 1.0, 1.0),                // top-right
            (page.x, page.y - page.height, -1.0, -1.0),             // bottom-left
            (page.x + page.width, page.y - page.height, 1.0, -1.0), // bottom-right
        ];
        for (x, y, ndc_x, ndc_y) in corners {
            let clip = view_proj * cgmath::Vector4::new(x, y, 0.0, 1.0);
            assert!(
                (clip.x - ndc_x).abs() < 1e-3 && (clip.y - ndc_y).abs() < 1e-3,
                "({x}, {y}) should map to ({ndc_x}, {ndc_y}), got ({}, {})",
                clip.x,
                clip.y
            );
        }
    }

    fn aabb(half_width: f32, half_height: f32) -> Aabb3 {
        Aabb3 {
            min: cgmath::Vector3::new(1.0 - half_width, 2.0 - half_height, 0.0),
            max: cgmath::Vector3::new(1.0 + half_width, 2.0 + half_height, 0.0),
        }
    }

    #[test]
    fn framing_centers_and_fits_the_box() {
        let camera = OrthographicCamera::default();
        // A wide box in a portrait viewport is width-bound; a tall box in a
        // landscape viewport is height-bound. Both must fit.
        for (box_, aspect) in [(aabb(3.0, 0.5), 0.5), (aabb(0.5, 3.0), 2.0)] {
            let (eye, zoom) = camera.frame_destination(&box_, aspect, 1.0).unwrap();
            assert_eq!(eye, cgmath::Point2::new(1.0, 2.0));
            let size = box_.size();
            assert!(1.0 / zoom >= size.y / 2.0, "zoom {zoom} clips the box vertically");
            assert!(aspect / zoom >= size.x / 2.0, "zoom {zoom} clips the box horizontally");
        }
    }

    #[test]
    fn a_tiny_selection_clamps_to_max_zoom() {
        let camera = OrthographicCamera::default();
        let point = cgmath::Vector3::new(1.0, 2.0, 0.0);
        let (_, zoom) =
            camera.frame_destination(&Aabb3 { min: point, max: point }, 1.0, 1.0).unwrap();
        assert_eq!(zoom, ORTHO_MAX_ZOOM);
    }

    #[test]
    fn an_empty_selection_has_nothing_to_frame() {
        let camera = OrthographicCamera::default();
        assert!(camera.frame_destination(&Aabb3::EMPTY, 1.0, 1.0).is_none());
    }

    #[test]
    fn animation_lands_on_the_destination_and_clears() {
        let box_ = aabb(2.0, 2.0);
        let mut camera = OrthographicCamera::default();
        let (eye, zoom) = camera.frame_destination(&box_, 1.0, 1.0).unwrap();
        camera.animate_to_frame(&box_, 1.0, 1.0);
        // The full duration at 60fps, plus a frame of overshoot
        for _ in 0..(FRAME_DURATION_MS / 16.0) as usize + 1 {
            camera.tick(16.0);
        }
        assert!(camera.animation.is_none(), "animation should have cleared");
        assert_eq!(camera.eye, eye);
        assert!((camera.zoom - zoom).abs() < 1e-4, "zoom {} should reach {zoom}", camera.zoom);
    }

    #[test]
    fn camera_input_cancels_the_animation() {
        let box_ = aabb(2.0, 2.0);
        let inputs: [(&str, fn(&mut OrthographicCamera)); 2] =
            [("pan", |c| c.pan(&cgmath::Point2::new(4.0, 4.0))), ("zoom", |c| c.zoom(1.0, 1.0))];
        for (name, input) in inputs {
            let mut camera = OrthographicCamera::default();
            camera.animate_to_frame(&box_, 1.0, 1.0);
            camera.tick(16.0);
            assert!(camera.animation.is_some());
            input(&mut camera);
            assert!(camera.animation.is_none(), "{name} should have cancelled the animation");
        }
    }
}
