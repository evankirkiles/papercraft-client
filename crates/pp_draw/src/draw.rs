use pp_editor::state::SelectionMode;

use crate::{
    cache,
    engines::ink::InkEngine,
    gpu,
    select::{SelectManager, SelectionMask},
    Renderer,
};

impl<'window> Renderer<'window> {
    /// Draws the "folding" view of a viewport, plus any active tool in the viewport
    pub(crate) fn draw_folding(
        &self,
        selection_mode: &SelectionMode,
        is_xray: bool,
        render_pass: &mut wgpu::RenderPass,
    ) {
        let Renderer { draw_cache, engine_ink, engine_overlay, .. } = &self;
        engine_overlay.grid_circle.draw(&self.ctx, draw_cache, render_pass);
        draw_cache.materials.iter().for_each(|(id, mat)| {
            mat.bind(render_pass);
            draw_cache.meshes.values().for_each(|mesh| {
                mesh.bind_model(render_pass);
                engine_ink.draw_mesh_for_material(&self.ctx, render_pass, mesh, &id);
            });
        });
        draw_cache.meshes.values().for_each(|mesh| {
            mesh.bind_model(render_pass);
            engine_ink.draw_mesh(&self.ctx, selection_mode, render_pass, mesh, is_xray);
        });
        engine_overlay.bbox.draw(&self.ctx, render_pass);
        // self.draw_cutting(selection_mode, render_pass);
    }

    /// Draws the view of a "cutting" viewport, plus any active tool in the viewport
    pub(crate) fn draw_cutting(
        &self,
        selection_mode: &SelectionMode,
        render_pass: &mut wgpu::RenderPass,
    ) {
        let Renderer { draw_cache, engine_ink, engine_overlay, ctx, .. } = &self;
        engine_overlay.grid_rect.draw(ctx, &draw_cache.printing, render_pass);
        engine_overlay.page.draw(ctx, render_pass, &draw_cache.printing);
        Self::draw_pieces(ctx, draw_cache, engine_ink, selection_mode, render_pass);
    }

    /// Draws the unfolded pieces themselves - textured surfaces, then the ink
    /// annotations over them - and nothing else.
    ///
    /// Split out from [`Self::draw_cutting`] because this is exactly what ends
    /// up on paper: the print pass draws the same call with no grid, no page
    /// backdrop and no tool overlay, and the two must not drift apart.
    ///
    /// Printing rasterizes the lines here rather than stroking them into the PDF
    /// as vector, which is what keeps the two identical for free. It also keeps
    /// the depth test - and so a tab tucked under the face that overlaps it,
    /// which paint order alone cannot express. See `docs/cut-contours.md` for the
    /// machine-readable cut geometry, which is computed rather than drawn.
    ///
    /// It takes its pieces of the renderer explicitly rather than `&self` so the
    /// print pass can substitute its own [`InkEngine`] - the sample count is
    /// baked into every pipeline, and print renders without MSAA.
    pub(crate) fn draw_pieces(
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        engine_ink: &InkEngine,
        selection_mode: &SelectionMode,
        render_pass: &mut wgpu::RenderPass,
    ) {
        draw_cache.materials.iter().for_each(|(id, mat)| {
            mat.bind(render_pass);
            draw_cache.meshes.values().for_each(|mesh| {
                engine_ink.draw_piece_mesh_for_material(ctx, render_pass, mesh, &id);
            });
        });
        draw_cache.meshes.values().for_each(|mesh| {
            engine_ink.draw_piece_mesh(ctx, selection_mode, render_pass, mesh);
        });
    }
}

impl SelectManager {
    pub(crate) fn draw_folding(
        &self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        mask: SelectionMask,
        is_xray: bool,
        render_pass: &mut wgpu::RenderPass,
    ) {
        draw_cache.meshes.values().for_each(|mesh| {
            mesh.bind_model(render_pass);
            self.select_engine.draw_mesh(ctx, render_pass, mesh, mask, is_xray);
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
