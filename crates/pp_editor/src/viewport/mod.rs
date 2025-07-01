use crate::measures::Rect;

pub mod camera;
pub mod cutting;
pub mod folding;

#[derive(Debug, Clone)]
pub enum ViewportContent {
    Folding(folding::FoldingViewport),
    Cutting(cutting::CuttingViewport),
}

impl Default for ViewportContent {
    fn default() -> Self {
        Self::Folding(Default::default())
    }
}

#[derive(Debug, Clone)]
pub struct ViewportBounds {
    /// The actual calculated area of the viewport based on the current window size
    pub area: Rect<f32>,
    /// The editor's DPR.
    pub dpr: f32,
}

/// A viewport represents a split of the window
#[derive(Debug, Clone)]
pub struct Viewport {
    /// The actual calculated area of the viewport based on the current window size
    pub bounds: ViewportBounds,
    /// The interior state of the viewport
    pub content: ViewportContent,
}
