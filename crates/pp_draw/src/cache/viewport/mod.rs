use camera::CameraGPU;
use pp_core::viewport::Viewport;

use super::prelude::*;
use crate::gpu;

mod camera;

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
pub struct ViewportGPU {
    pub camera: camera::CameraGPU,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ViewportGPU {
    /// Sets the render viewport and binds the camera bind group
    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_viewport(self.x, self.y, self.width, self.height, 0.0, 1.0);
        self.camera.bind(render_pass);
    }
}

impl GPUCache<Viewport> for ViewportGPU {
    fn new(ctx: &gpu::Context, viewport: &Viewport) -> Self {
        let mut vp = Self { camera: CameraGPU::new(ctx), x: 0.0, y: 0.0, width: 1.0, height: 1.0 };
        vp.sync(ctx, viewport);
        vp
    }

    fn sync(&mut self, ctx: &gpu::Context, viewport: &pp_core::viewport::Viewport) {
        self.width = viewport.width_frac * ctx.config.width as f32;
        self.height = ctx.config.height as f32;
        self.camera.sync(ctx, &viewport.camera, self.width, self.height);
    }
}
