use crate::gpu;

mod camera;
use camera::Camera3DGPU;
use pp_core::viewport::Viewport3D;

use super::{ViewportGPU, ViewportNotVisible};

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
#[derive(Debug)]
pub struct Viewport3DGPU {
    pub camera: camera::Camera3DGPU,
    pub xray_mode: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Viewport3DGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            camera: Camera3DGPU::new(ctx),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            xray_mode: false,
        }
    }
}

impl ViewportGPU<Viewport3D> for Viewport3DGPU {
    /// Sets the render viewport and binds the camera bind group
    fn bind(&self, render_pass: &mut wgpu::RenderPass) -> Result<(), ViewportNotVisible> {
        if self.width == 0.0 || self.height == 0.0 {
            return Err(ViewportNotVisible);
        }
        render_pass.set_viewport(self.x, self.y, self.width, self.height, 0.0, 1.0);
        self.camera.bind(render_pass);
        Ok(())
    }

    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &Viewport3D,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.camera.sync(ctx, &viewport.camera, self.width, self.height);
        self.xray_mode = viewport.xray_mode;
    }
}
