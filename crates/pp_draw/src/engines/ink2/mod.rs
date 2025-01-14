use crate::gpu;

use super::program::Drawable;

mod overlay_grid;

pub struct InkEngine2D {
    program_overlay_grid: overlay_grid::Program,
}

impl InkEngine2D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { program_overlay_grid: overlay_grid::Program::new(ctx) }
    }

    pub fn draw_overlays(&self, render_pass: &mut wgpu::RenderPass) {
        self.program_overlay_grid.draw(render_pass);
    }
}
