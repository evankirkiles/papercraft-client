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
