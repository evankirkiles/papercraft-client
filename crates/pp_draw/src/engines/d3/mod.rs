use crate::{cache, gpu};

use super::program::{Drawable, MeshDrawable};

mod cut_lines;
mod lines;
mod overlay_grid;
mod points;
mod surface;
mod tris;

#[derive(Debug)]
pub struct InkEngine3D {
    // Mesh draw programs
    program_surface: surface::Program,
    program_lines: lines::Program,
    program_cut_lines: cut_lines::Program,
    program_points: points::Program,
    program_tris: tris::Program,
    // Overlay draw programs
    program_overlay_grid: overlay_grid::Program,
}

impl InkEngine3D {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            program_surface: surface::Program::new(ctx),
            program_lines: lines::Program::new(ctx),
            program_cut_lines: cut_lines::Program::new(ctx),
            program_points: points::Program::new(ctx),
            program_tris: tris::Program::new(ctx),
            program_overlay_grid: overlay_grid::Program::new(ctx),
        }
    }

    pub fn draw_overlays(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        self.program_overlay_grid.draw(ctx, render_pass);
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        xray: bool,
    ) {
        self.program_surface.draw_mesh(ctx, render_pass, mesh);

        if xray {
            // occluded wireframe elements go over the surface in xray mode
            self.program_tris.draw_mesh_xrayed(ctx, render_pass, mesh);
            self.program_lines.draw_mesh_xrayed(ctx, render_pass, mesh);
            self.program_cut_lines.draw_mesh_xrayed(ctx, render_pass, mesh);
            self.program_points.draw_mesh_xrayed(ctx, render_pass, mesh);
        };

        // always draw non-occluded elements
        self.program_tris.draw_mesh(ctx, render_pass, mesh);
        self.program_lines.draw_mesh(ctx, render_pass, mesh);
        self.program_cut_lines.draw_mesh(ctx, render_pass, mesh);
        self.program_points.draw_mesh(ctx, render_pass, mesh);
    }
}
