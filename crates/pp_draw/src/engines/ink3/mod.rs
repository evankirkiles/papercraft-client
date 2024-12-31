use crate::{cache, gpu};

mod prg_edge;
mod prg_surface;

pub struct InkEngine3D {
    program_surface: prg_surface::ProgramSurface,
    program_edge: prg_edge::ProgramEdge,
}

impl InkEngine3D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            program_surface: prg_surface::ProgramSurface::new(ctx),
            program_edge: prg_edge::ProgramEdge::new(ctx),
        }
    }

    pub fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU) {
        self.program_surface.draw_mesh(render_pass, mesh);
        self.program_edge.draw_mesh(render_pass, mesh);
    }
}
