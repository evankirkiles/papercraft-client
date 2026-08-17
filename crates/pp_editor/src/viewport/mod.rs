use serde::Serialize;
use tsify::Tsify;

use pp_core::measures::Rect;

pub mod camera;
pub mod cutting;
pub mod folding;

#[derive(Debug, Clone)]
pub enum ViewportContent {
    Folding(folding::FoldingViewport),
    Cutting(cutting::CuttingViewport),
}

impl serde::Serialize for ViewportContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant_name = match self {
            ViewportContent::Folding(_) => "Folding",
            ViewportContent::Cutting(_) => "Cutting",
        };
        serializer.serialize_str(variant_name)
    }
}

impl Default for ViewportContent {
    fn default() -> Self {
        Self::Folding(Default::default())
    }
}

#[derive(Debug, Clone, Tsify, Serialize)]
pub struct ViewportBounds {
    /// The actual calculated area of the viewport based on the current window size
    pub area: Rect<f32>,
    /// The editor's DPR.
    pub dpr: f32,
    /// Does this viewport bound's GPU representation need to be updated
    pub is_dirty: bool,
}

/// A viewport represents a split of the window
#[derive(Debug, Clone, Tsify, Serialize)]
pub struct Viewport {
    /// The actual calculated area of the viewport based on the current window size
    pub bounds: ViewportBounds,
    /// The interior state of the viewport
    #[tsify(type = "\"Folding\" | \"Cutting\"")]
    pub content: ViewportContent,
}

impl Viewport {
    /// Whether this viewport's camera has moved since it was last synced to the GPU
    pub fn camera_is_dirty(&self) -> bool {
        use camera::Camera;
        match &self.content {
            ViewportContent::Folding(viewport) => viewport.camera.is_dirty(),
            ViewportContent::Cutting(viewport) => viewport.camera.is_dirty(),
        }
    }

    /// Advances this viewport camera's in-flight framing move, if any.
    pub fn tick_camera(&mut self, dt_ms: f32) {
        match &mut self.content {
            ViewportContent::Folding(viewport) => viewport.camera.tick(dt_ms),
            ViewportContent::Cutting(viewport) => viewport.camera.tick(dt_ms),
        }
    }

    /// The viewport's width over its height, as used by `Camera::view_proj`.
    pub fn aspect(&self) -> f32 {
        self.bounds.area.width.max(1.0) / self.bounds.area.height.max(1.0)
    }
}
