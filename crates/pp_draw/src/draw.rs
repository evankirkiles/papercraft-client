use pp_core::settings::Settings;

use crate::{
    cache, gpu,
    select::{SelectManager, SelectionMask},
    Renderer,
};

impl<'window> Renderer<'window> {
    /// Draws the "folding" view of a viewport, plus any active tool in the viewport
    pub(crate) fn draw_folding(&self, settings: &Settings, render_pass: &mut wgpu::RenderPass) {
        let Renderer { draw_cache, engine_ink, engine_overlay, .. } = &self;
        engine_overlay.grid_circle.draw(&self.ctx, render_pass);
        draw_cache.common.piece_identity.bind(render_pass);
        draw_cache.materials.iter().for_each(|(id, mat)| {
            mat.bind(render_pass);
            draw_cache.meshes.values().for_each(|mesh| {
                engine_ink.draw_mesh_for_material(&self.ctx, render_pass, mesh, &id);
            });
        });
        let xray_mode = false;
        draw_cache.meshes.values().for_each(|mesh| {
            engine_ink.draw_mesh(&self.ctx, settings, render_pass, mesh, xray_mode);
        });
        self.draw_cutting(settings, render_pass);
    }

    /// Draws the view of a "cutting" viewport, plus any active tool in the viewport
    pub(crate) fn draw_cutting(&self, settings: &Settings, render_pass: &mut wgpu::RenderPass) {
        let Renderer { draw_cache, engine_ink, engine_overlay, ctx, .. } = &self;
        engine_overlay.grid_rect.draw(&self.ctx, render_pass);
        engine_overlay.page.draw(ctx, render_pass, &draw_cache.printing);
        draw_cache.materials.iter().for_each(|(id, mat)| {
            mat.bind(render_pass);
            draw_cache.meshes.values().for_each(|mesh| {
                engine_ink.draw_piece_mesh_for_material(ctx, render_pass, mesh, &id);
            });
        });
        draw_cache.meshes.values().for_each(|mesh| {
            engine_ink.draw_piece_mesh(ctx, settings, render_pass, mesh);
        });
    }
}

impl SelectManager {
    pub(crate) fn draw_folding(
        &self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        mask: SelectionMask,
        render_pass: &mut wgpu::RenderPass,
    ) {
        draw_cache.common.piece_identity.bind(render_pass);
        let xray_mode = false;
        draw_cache.meshes.values().for_each(|mesh| {
            self.select_engine.draw_mesh(ctx, render_pass, mesh, mask, xray_mode);
        });
    }

    pub(crate) fn draw_cutting(
        &self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        mask: SelectionMask,
        render_pass: &mut wgpu::RenderPass,
    ) {
        draw_cache.meshes.values().for_each(|mesh| {
            self.select_engine.draw_piece_mesh(ctx, render_pass, mesh, mask);
        });
    }
}
