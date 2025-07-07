use grid_circle::GridCircleProgram;
use grid_rect::GridRectProgram;
use page::PageProgram;
use tool_rotate::ToolRotateProgram;
use tool_select_box::ToolSelectBoxProgram;
use tool_translate::ToolTranslateProgram;

use crate::{cache::tool::ToolGPU, gpu};

pub mod grid_circle;
pub mod grid_rect;
pub mod page;
pub mod tool_rotate;
pub mod tool_select_box;
pub mod tool_translate;

#[derive(Debug)]
pub struct OverlayEngine {
    tool_select_box: ToolSelectBoxProgram,
    tool_rotate: ToolRotateProgram,
    tool_translate: ToolTranslateProgram,

    pub page: PageProgram,
    pub grid_circle: GridCircleProgram,
    pub grid_rect: GridRectProgram,
}

impl OverlayEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            page: PageProgram::new(ctx),
            tool_select_box: ToolSelectBoxProgram::new(ctx),
            tool_rotate: ToolRotateProgram::new(ctx),
            tool_translate: ToolTranslateProgram::new(ctx),
            grid_circle: GridCircleProgram::new(ctx),
            grid_rect: GridRectProgram::new(ctx),
        }
    }

    pub fn draw_tool(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        tool: &ToolGPU,
    ) {
        match tool {
            ToolGPU::SelectBox(tool) => self.tool_select_box.draw(ctx, render_pass, tool),
            ToolGPU::Rotate(tool) => self.tool_rotate.draw(ctx, render_pass, tool),
            ToolGPU::Translate(tool) => self.tool_translate.draw(ctx, render_pass, tool),
        }
    }
}
