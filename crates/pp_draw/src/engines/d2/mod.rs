use crate::gpu;

mod overlay_grid;

#[derive(Debug)]
pub struct InkEngine2D {
    program_overlay_grid: overlay_grid::Program,
}

impl InkEngine2D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { program_overlay_grid: overlay_grid::Program::new(ctx) }
    }

    pub fn draw_overlays(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        self.program_overlay_grid.draw(ctx, render_pass);
    }
}
