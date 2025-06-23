use crate::PhysicalDimensions;

pub mod d2;

/// An error indicating a tool could not be created.
#[derive(Debug, Clone, Copy)]
pub enum ToolCreationError {
    NoSelection,
}

/// Common fields provided by the controller which all tools may need
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolContext {
    /// The raw pixel dimensions of the viewport this tool operates on
    pub viewport: PhysicalDimensions<f32>,
    /// The Device Pixel Ratio, the number of pixels per logical pixel
    pub dpr: f32,
}
