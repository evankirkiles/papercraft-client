use pp_core::measures::Rect;
use pp_editor::viewport::{Viewport, ViewportContent};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

use super::{bounds::ViewportBoundsGPU, camera::CameraGPU, BindableViewport, ViewportSyncError};

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
#[derive(Debug, Clone)]
pub struct FoldingViewportGPU {
    viewport: ViewportBoundsGPU,
    camera: CameraGPU,
}

impl FoldingViewportGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { viewport: ViewportBoundsGPU::new(ctx), camera: CameraGPU::new(ctx) }
    }
}

impl BindableViewport for FoldingViewportGPU {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError> {
        let ViewportContent::Folding(fold_viewport) = &mut viewport.content else {
            return Err(ViewportSyncError::WrongViewportType);
        };
        self.camera.sync(ctx, &mut fold_viewport.camera, &viewport.bounds);
        self.viewport.sync(ctx, viewport);
        Ok(())
    }

    fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        let Rect { x, y, width, height } = self.viewport.area;
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.set_bind_group(BindGroup::Viewport.value(), &self.viewport.bind_group, &[]);
        render_pass.set_bind_group(BindGroup::Camera.value(), &self.camera.bind_group, &[]);
    }
}
