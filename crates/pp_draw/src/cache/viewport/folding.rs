use pp_editor::{
    measures::Rect,
    viewport::{Viewport, ViewportContent},
};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

use super::{
    bounds::ViewportBoundsBindGroup, camera::CameraBindGroup, BindableViewport, ViewportSyncError,
};

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
#[derive(Debug, Clone)]
pub struct FoldingViewportGPU {
    viewport: ViewportBoundsBindGroup,
    camera: CameraBindGroup,
}

impl FoldingViewportGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { viewport: ViewportBoundsBindGroup::new(ctx), camera: CameraBindGroup::new(ctx) }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport.folding"),
            entries: &[wgpu::BindGroupLayoutEntry {
                count: None,
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }
}

impl BindableViewport for FoldingViewportGPU {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError> {
        self.viewport.sync(ctx, viewport);
        let ViewportContent::Folding(fold_viewport) = &mut viewport.content else {
            return Err(ViewportSyncError::WrongViewportType);
        };
        self.camera.sync(ctx, &mut fold_viewport.camera, &self.viewport.area);
        Ok(())
    }

    fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        let Rect { x, y, width, height } = self.viewport.area;
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.set_bind_group(BindGroup::Viewport.value(), &self.viewport.bind_group, &[]);
        render_pass.set_bind_group(BindGroup::Camera.value(), &self.camera.bind_group, &[]);
    }
}
