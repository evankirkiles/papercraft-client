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
}
