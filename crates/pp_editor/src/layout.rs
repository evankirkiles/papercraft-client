use pp_core::measures::Dimensions;
use serde::Serialize;
use slotmap::SlotMap;
use tsify::Tsify;

use crate::{
    viewport::Viewport,
    windowing::{Split, ViewTreeNode},
    SplitId, ViewportId,
};

/// The window's layout: its recursive tree of splits/viewports, plus the
/// current screen dimensions driving that tree's sizing.
#[derive(Debug, Tsify, Serialize)]
pub struct Layout {
    /// The current dimensions of the editor
    pub dimensions: Dimensions<f32>,
    /// The DPI of the editor
    pub dpr: f32,
    /// The window's full recursive tree layout, e.g. splits and viewports
    pub root_node: ViewTreeNode,
    /// Cuts where a node is split into separate viewports
    pub splits: SlotMap<SplitId, Split>,
    /// The leaves of the editor node tree where content is actually rendered
    pub viewports: SlotMap<ViewportId, Viewport>,
}
