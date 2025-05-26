use pp_core::settings::SelectionMode;

use crate::{cache, gpu};

mod lines;
mod lines_cut;
mod overlay_grid_circle;
mod overlay_grid_rect;
mod points;
mod surface;
mod tris;

/// Constant layers
#[repr(i32)]
#[derive(Default)]
pub enum DepthBiasLayer {
    ForegroundTop,
    ForegroundMiddle,
    ForegroundBottom,
    #[default]
    Default,
    BackgroundTop,
    BackgroundMiddle,
    BackgroundBottom,
}

#[derive(Debug)]
pub struct InkEngine {
    // Mesh draw programs
    points: points::PointsProgram,
    lines: lines::LinesProgram,
    lines_cut: lines_cut::LinesCutProgram,
    tris: tris::TrisProgram,
    surface: surface::SurfaceProgram,

    // Overlay draw programs
    overlay_grid_sphere: overlay_grid_circle::OverlayGridCircleProgram,
    overlay_grid_rect: overlay_grid_rect::OverlayGridRectProgram,
}

impl InkEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            surface: surface::SurfaceProgram::new(ctx),
            lines: lines::LinesProgram::new(ctx),
            lines_cut: lines_cut::LinesCutProgram::new(ctx),
            points: points::PointsProgram::new(ctx),
            tris: tris::TrisProgram::new(ctx),
            overlay_grid_sphere: overlay_grid_circle::OverlayGridCircleProgram::new(ctx),
            overlay_grid_rect: overlay_grid_rect::OverlayGridRectProgram::new(ctx),
        }
    }

    pub fn draw_3d_overlays(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        self.overlay_grid_sphere.draw(ctx, render_pass);
    }

    pub fn draw_2d_overlays(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        self.overlay_grid_rect.draw(ctx, render_pass);
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        settings: &pp_core::settings::Settings,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        xray: bool,
    ) {
        self.surface.draw_mesh(ctx, render_pass, mesh);

        if xray {
            // occluded wireframe elements go over the surface in xray mode
            self.tris.draw_mesh_xrayed(ctx, render_pass, mesh);
            self.lines.draw_mesh_xrayed(ctx, render_pass, mesh);
            self.lines_cut.draw_mesh_xrayed(ctx, render_pass, mesh);
            if settings.selection_mode == SelectionMode::Vert {
                self.points.draw_mesh_xrayed(ctx, render_pass, mesh);
            }
        };

        // always draw non-occluded elements
        self.tris.draw_mesh(ctx, render_pass, mesh);
        self.lines.draw_mesh(ctx, render_pass, mesh);
        self.lines_cut.draw_mesh(ctx, render_pass, mesh);
        if settings.selection_mode == SelectionMode::Vert {
            self.points.draw_mesh(ctx, render_pass, mesh);
        }
    }

    pub fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        settings: &pp_core::settings::Settings,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        self.surface.draw_piece_mesh(ctx, render_pass, mesh);
        self.tris.draw_piece_mesh(ctx, render_pass, mesh);
        self.lines.draw_piece_mesh(ctx, render_pass, mesh);
        if settings.selection_mode == SelectionMode::Vert {
            self.points.draw_piece_mesh(ctx, render_pass, mesh);
        }
    }
}
