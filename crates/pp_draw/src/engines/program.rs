use crate::{cache, gpu};

pub trait Drawable {
    fn new(ctx: &gpu::Context) -> Self;
    fn draw(&self, render_pass: &mut wgpu::RenderPass);
}

pub trait MeshDrawable {
    fn new(ctx: &gpu::Context) -> Self;
    fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU);
}
