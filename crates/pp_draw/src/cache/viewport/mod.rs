use camera::CameraGPU;

use crate::gpu;
mod camera;

pub struct ViewportGPU {
    camera: camera::CameraGPU,

    /// Viewport dimensions
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ViewportGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { camera: CameraGPU::new(ctx), x: 0.0, y: 0.0, width: 1.0, height: 1.0 }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, viewport: &pp_core::viewport::Viewport) {
        self.width = viewport.width_frac * ctx.config.width as f32;
        self.height = ctx.config.height as f32;
        self.camera.sync(ctx, &viewport.camera, self.width / self.height);
    }

    /// Sets the render viewport and binds the camera bind group
    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_viewport(self.x, self.y, self.width, self.height, 0.0, 1.0);
        self.camera.bind(render_pass);
    }
}
