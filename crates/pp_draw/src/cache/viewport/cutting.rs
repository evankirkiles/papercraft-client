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
pub struct CuttingViewportGPU {
    viewport: ViewportBoundsBindGroup,
    camera: CameraBindGroup,
}

impl CuttingViewportGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport.cutting"),
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

    pub fn new(ctx: &gpu::Context) -> Self {
        Self { viewport: ViewportBoundsBindGroup::new(ctx), camera: CameraBindGroup::new(ctx) }
    }
}

impl BindableViewport for CuttingViewportGPU {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError> {
        self.viewport.sync(ctx, viewport);
        let ViewportContent::Cutting(cut_viewport) = &mut viewport.content else {
            return Err(ViewportSyncError::WrongViewportType);
        };
        self.camera.sync(ctx, &mut cut_viewport.camera, &self.viewport.area);
        Ok(())
    }

    fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        let Rect { x, y, width, height } = self.viewport.area;
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.set_bind_group(BindGroup::Viewport.value(), &self.viewport.bind_group, &[]);
        render_pass.set_bind_group(BindGroup::Camera.value(), &self.camera.bind_group, &[]);
    }
}
