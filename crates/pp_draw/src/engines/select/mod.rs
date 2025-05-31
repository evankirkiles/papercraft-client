use crate::{cache, gpu, select};

mod lines;
mod points;
mod surface;
mod tris;

#[derive(Debug)]
pub(crate) struct SelectEngine {
    points: points::PointsProgram,
    lines: lines::LinesProgram,
    tris: tris::TrisProgram,
    surface: surface::SurfaceProgram,
}

impl SelectEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            points: points::PointsProgram::new(ctx),
            lines: lines::LinesProgram::new(ctx),
            tris: tris::TrisProgram::new(ctx),
            surface: surface::SurfaceProgram::new(ctx),
        }
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        mask: select::SelectionMask,
        xray: bool,
    ) {
        if mask.intersects(select::SelectionMask::VERTS) {
            self.points.draw_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::EDGES) {
            self.lines.draw_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::FACES | select::SelectionMask::PIECES) {
            self.tris.draw_mesh(ctx, render_pass, mesh);
        }
        if !xray {
            self.surface.draw_mesh(ctx, render_pass, mesh);
        }
    }

    pub fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        mask: select::SelectionMask,
    ) {
        if mask.intersects(select::SelectionMask::VERTS) {
            self.points.draw_piece_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::EDGES) {
            self.lines.draw_piece_mesh(ctx, render_pass, mesh);
        }
        if mask.intersects(select::SelectionMask::FACES | select::SelectionMask::PIECES) {
            self.tris.draw_piece_mesh(ctx, render_pass, mesh);
        }
    }
}
