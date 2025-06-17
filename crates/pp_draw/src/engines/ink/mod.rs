use pp_core::{id, settings::SelectionMode};

use crate::{cache, gpu};

mod flaps;
mod flaps_lines;
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
    flaps: flaps::FlapsProgram,
    flaps_lines: flaps_lines::FlapsLinesProgram,

    // Overlay draw programs
    overlay_grid_sphere: overlay_grid_circle::OverlayGridCircleProgram,
    overlay_grid_rect: overlay_grid_rect::OverlayGridRectProgram,
}

impl InkEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            lines: lines::LinesProgram::new(ctx),
            lines_cut: lines_cut::LinesCutProgram::new(ctx),
            points: points::PointsProgram::new(ctx),
            tris: tris::TrisProgram::new(ctx),
            surface: surface::SurfaceProgram::new(ctx),
            flaps: flaps::FlapsProgram::new(ctx),
            flaps_lines: flaps_lines::FlapsLinesProgram::new(ctx),
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

    /// Draws only the parts of the mesh using the specified material
    pub fn draw_mesh_for_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &id::MaterialId,
    ) {
        self.surface.draw_mesh_with_material(ctx, render_pass, mesh, material_id);
    }

    pub fn draw_piece_mesh_for_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &id::MaterialId,
    ) {
        self.surface.draw_piece_mesh_with_material(ctx, render_pass, mesh, material_id);
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        settings: &pp_core::settings::Settings,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        xray: bool,
    ) {
        if xray {
            // occluded wireframe elements go over the surface in xray mode
            if settings.selection_mode == SelectionMode::Vert {
                self.points.draw_mesh_xrayed(ctx, render_pass, mesh);
            }
            self.lines_cut.draw_mesh_xrayed(ctx, render_pass, mesh);
            if settings.selection_mode != SelectionMode::Piece {
                self.lines.draw_mesh_xrayed(ctx, render_pass, mesh);
            }
            self.tris.draw_mesh_xrayed(ctx, render_pass, mesh);
        };

        // always draw non-occluded elements
        self.tris.draw_mesh(ctx, render_pass, mesh);
        if settings.selection_mode != SelectionMode::Piece {
            self.lines.draw_mesh(ctx, render_pass, mesh);
        }
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
        self.tris.draw_piece_mesh(ctx, render_pass, mesh);
        if settings.selection_mode != SelectionMode::Piece {
            self.lines.draw_piece_mesh(ctx, render_pass, mesh);
        }
        self.flaps.draw_piece_mesh(ctx, render_pass, mesh);
        self.flaps_lines.draw_piece_mesh(ctx, render_pass, mesh);
        if settings.selection_mode == SelectionMode::Vert {
            self.points.draw_piece_mesh(ctx, render_pass, mesh);
        }
    }
}
