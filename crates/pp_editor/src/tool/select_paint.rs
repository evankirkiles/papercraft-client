use pp_core::select::{SelectionActionType, SelectionState};

/// The default brush radius, in logical (pre-DPR) pixels
pub const DEFAULT_RADIUS: f32 = 40.0;
/// The smallest brush radius, in logical (pre-DPR) pixels
pub const MIN_RADIUS: f32 = 4.0;
/// The largest brush radius, in logical (pre-DPR) pixels
pub const MAX_RADIUS: f32 = 400.0;

/// Continuously selects (or deselects) every element the brush circle sweeps
/// over. Unlike the other tools, this one is a persistent mode: it stays active
/// until the user cancels it, and swallows all input while it is.
#[derive(Debug, Clone)]
pub struct SelectPaintTool {
    /// The center of the brush circle, in surface pixels
    pub cursor_pos: cgmath::Point2<f32>,
    /// The radius of the brush circle, in surface pixels
    pub radius: f32,
    /// The action being applied, present only while a stroke is in progress
    pub action: Option<SelectionActionType>,
    /// The selection as it was when the current stroke began, so the whole
    /// stroke can be pushed onto the history stack as a single entry
    pub stroke_start: Option<Box<SelectionState>>,
    /// Indicates the tool's state has changed
    pub is_dirty: bool,
}

impl SelectPaintTool {
    pub fn new(cursor_pos: cgmath::Point2<f32>, radius: f32) -> Self {
        Self { cursor_pos, radius, action: None, stroke_start: None, is_dirty: true }
    }

    pub fn update(&mut self, pos: cgmath::Point2<f32>) {
        self.cursor_pos = pos;
        self.is_dirty = true;
    }

    /// Grows / shrinks the brush, clamped to the allowed range. `dpi` scales the
    /// logical-pixel bounds into the surface pixels the radius is stored in.
    pub fn set_radius(&mut self, radius: f32, dpi: f32) {
        self.radius = radius.clamp(MIN_RADIUS * dpi, MAX_RADIUS * dpi);
        self.is_dirty = true;
    }
}
