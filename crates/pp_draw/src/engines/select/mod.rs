use super::program::MeshDrawable;
use crate::{cache, gpu, select};

mod lines;
mod points;

#[derive(Debug)]
pub(crate) struct SelectEngine {
    program_points: points::Program,
    program_lines: lines::Program,
}

impl SelectEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { program_points: points::Program::new(ctx), program_lines: lines::Program::new(ctx) }
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        mask: select::SelectionMask,
    ) {
        if mask.intersects(select::SelectionMask::POINTS) {
            self.program_points.draw_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::LINES) {
            self.program_lines.draw_mesh(ctx, render_pass, mesh);
        }
        // if mask.intersects(select::SelectionMask::FACES) {
        //     self.program_points.draw_mesh(ctx, render_pass, mesh);
        // }
    }
}
