use pp_editor::{tool::Tool, ViewportId};
use rotate::RotateToolGPU;
use select_box::SelectBoxToolGPU;
use select_paint::SelectPaintToolGPU;
use translate::TranslateToolGPU;

use crate::gpu;

pub mod rotate;
pub mod select_box;
pub mod select_paint;
pub mod translate;

/// Represents the currently-bound tool in the viewport
#[derive(Debug)]
pub struct ActiveToolGPU {
    /// The viewport this tool is active in (it is only drawn in this viewport)
    pub viewport: ViewportId,
    /// The GPU resources for this tool
    pub tool: ToolGPU,
}

#[derive(Debug)]
pub enum ToolGPU {
    SelectBox(SelectBoxToolGPU),
    SelectPaint(SelectPaintToolGPU),
    Rotate(RotateToolGPU),
    Translate(TranslateToolGPU),
}

impl ToolGPU {
    /// Creates a new GPU tool context
    pub fn new(ctx: &gpu::Context, tool: &Tool) -> Self {
        match tool {
            Tool::SelectBox(tool) => Self::SelectBox(SelectBoxToolGPU::new(ctx, tool)),
            Tool::SelectPaint(tool) => Self::SelectPaint(SelectPaintToolGPU::new(ctx, tool)),
            Tool::Rotate(tool) => Self::Rotate(RotateToolGPU::new(ctx, tool)),
            Tool::Translate(tool) => Self::Translate(TranslateToolGPU::new(ctx, tool)),
        }
    }
}
