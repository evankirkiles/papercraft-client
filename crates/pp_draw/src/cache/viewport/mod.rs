use crate::gpu;

mod d2;
mod d3;

pub use d2::*;
pub use d3::*;

pub struct ViewportNotVisible;

pub trait ViewportGPU<T> {
    fn bind(&self, render_pass: &mut wgpu::RenderPass) -> Result<(), ViewportNotVisible>;
    fn sync(&mut self, ctx: &gpu::Context, viewport: &T, x: f32, y: f32, width: f32, height: f32);
}
