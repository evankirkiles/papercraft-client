use crate::{cache, gpu};

mod prg_lines;
mod prg_points;
mod prg_surface;

pub struct InkEngine3D {
    program_surface: prg_surface::ProgramSurface,
    program_lines: prg_lines::ProgramLines,
    program_points: prg_points::ProgramPoints,
}

impl InkEngine3D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            program_surface: prg_surface::ProgramSurface::new(ctx),
            program_lines: prg_lines::ProgramLines::new(ctx),
            program_points: prg_points::ProgramPoints::new(ctx),
        }
    }

    pub fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU) {
        self.program_surface.draw_mesh(render_pass, mesh);
        self.program_lines.draw_mesh(render_pass, mesh);
        self.program_points.draw_mesh(render_pass, mesh);
    }
}
