use crate::{cache, gpu};

use super::program::MeshDrawable;

use bitflags::bitflags;

mod lines;
mod points;

bitflags! {
    /// A mask of items to render for selection in the buffer
    #[derive(Debug, Clone, Copy)]
    pub struct SelectionMask: u8 {
        const POINTS = 1 << 0;
        const LINES = 1 << 1;
        const FACES = 1 << 2;
    }
}

pub struct SelectEngine {
    program_points: points::Program,
    program_lines: lines::Program,
}

impl SelectEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { program_points: points::Program::new(ctx), program_lines: lines::Program::new(ctx) }
    }

    pub fn draw_mesh(
        &self,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        mask: SelectionMask,
    ) {
        if mask.intersects(SelectionMask::LINES) {
            self.program_lines.draw_mesh(render_pass, mesh);
        }
        // if mask.intersects(SelectionMask::POINTS) {
        //     self.program_points.draw_mesh(render_pass, mesh);
        // }
    }
}
