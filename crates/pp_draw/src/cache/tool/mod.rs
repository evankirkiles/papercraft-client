use pp_editor::{tool::Tool, ViewportId};
use select_box::SelectBoxToolGPU;

use crate::gpu;

pub mod select_box;

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
}

impl ToolGPU {
    /// Creates a new GPU tool context
    pub fn new(ctx: &gpu::Context, tool: &Tool) -> Self {
        match tool {
            Tool::SelectBox(tool) => Self::SelectBox(SelectBoxToolGPU::new(ctx, tool)),
            Tool::Translate(tool) => todo!(),
            Tool::Rotate(tool) => todo!(),
        }
    }
}
