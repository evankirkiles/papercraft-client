use tool_select_box::ToolSelectBoxProgram;

use crate::{cache::tool::ToolGPU, gpu};

pub mod tool_select_box;

#[derive(Debug)]
pub struct OverlayEngine {
    tool_select_box: ToolSelectBoxProgram,
}

impl OverlayEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { tool_select_box: ToolSelectBoxProgram::new(ctx) }
    }

    pub fn draw_tool(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        tool: &ToolGPU,
    ) {
        match tool {
            ToolGPU::SelectBox(tool) => self.tool_select_box.draw(ctx, render_pass, tool),
        }
    }
}
