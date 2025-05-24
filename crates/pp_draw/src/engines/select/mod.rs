use crate::{cache, gpu, select};

mod lines;
mod points;

#[derive(Debug)]
pub(crate) struct SelectEngine {
    points: points::PointsProgram,
    lines: lines::LinesProgram,
}

impl SelectEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { points: points::PointsProgram::new(ctx), lines: lines::LinesProgram::new(ctx) }
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        mask: select::SelectionMask,
    ) {
        if mask.intersects(select::SelectionMask::POINTS) {
            self.points.draw_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::LINES) {
            self.lines.draw_mesh(ctx, render_pass, mesh);
        }
        // if mask.intersects(select::SelectionMask::FACES) {
        //     self.program_points.draw_mesh(ctx, render_pass, mesh);
        // }
    }
}
