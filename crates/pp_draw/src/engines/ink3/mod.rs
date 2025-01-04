use crate::{cache, gpu};

mod prg_lines;
mod prg_overlay_grid;
mod prg_points;
mod prg_surface;

pub struct InkEngine3D {
    program_surface: prg_surface::ProgramSurface,
    program_lines: prg_lines::ProgramLines,
    program_points: prg_points::ProgramPoints,
    program_overlay_grid: prg_overlay_grid::ProgramOverlayGrid,
}

impl InkEngine3D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            program_surface: prg_surface::ProgramSurface::new(ctx),
            program_lines: prg_lines::ProgramLines::new(ctx),
            program_points: prg_points::ProgramPoints::new(ctx),
            program_overlay_grid: prg_overlay_grid::ProgramOverlayGrid::new(ctx),
        }
    }

    pub fn draw_overlays(&self, render_pass: &mut wgpu::RenderPass) {
        self.program_overlay_grid.draw(render_pass);
    }

    pub fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU) {
        self.program_surface.draw_mesh(render_pass, mesh);
        self.program_lines.draw_mesh(render_pass, mesh);
        self.program_points.draw_mesh(render_pass, mesh);
    }
}
