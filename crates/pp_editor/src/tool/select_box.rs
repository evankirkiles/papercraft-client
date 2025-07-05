use pp_core::select::SelectionActionType;

/// Transforms the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct SelectBoxTool {
    /// The top-left point of the select box
    pub start_pos: cgmath::Point2<f32>,
    /// The bottom-right point of the select box
    pub end_pos: cgmath::Point2<f32>,
    /// The type of selection that will happen
    pub action: SelectionActionType,
    /// Indicates the tool's state has changed
    pub is_dirty: bool,
}

impl SelectBoxTool {
    pub fn update(&mut self, pos: cgmath::Point2<f32>) {
        self.end_pos = pos;
        self.is_dirty = true;
    }
}
