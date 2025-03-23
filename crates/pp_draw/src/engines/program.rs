use crate::{cache, gpu};

pub(crate) trait Drawable {
    fn new(ctx: &gpu::Context) -> Self;
    fn draw(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass);
}

pub(crate) trait MeshDrawable {
    fn new(ctx: &gpu::Context) -> Self;
    fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    );
}
