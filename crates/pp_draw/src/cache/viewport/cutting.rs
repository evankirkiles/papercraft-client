use std::mem;

use pp_editor::{
    measures::Rect,
    viewport::{Viewport, ViewportBounds, ViewportContent},
};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

use super::{camera::CameraUniform, BindableViewport, ViewportSyncError};

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
#[derive(Debug, Clone)]
pub struct CuttingViewportGPU {
    area: Rect<f32>,
    buf_camera: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
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
        let buf_camera =
            gpu::UniformBuf::new(ctx, "camera".to_string(), mem::size_of::<CameraUniform>());
        Self {
            area: Rect::default(),
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("viewport.cutting"),
                layout: &ctx.shared.bind_group_layouts.viewport_cutting,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_camera.binding_resource(),
                }],
            }),
            buf_camera,
        }
    }
}

impl BindableViewport for CuttingViewportGPU {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError> {
        let Viewport { bounds: ViewportBounds { area, .. }, content } = viewport;
        let ViewportContent::Cutting(viewport) = content else {
            return Err(ViewportSyncError::WrongViewportType);
        };
        if (self.area != *area) || viewport.camera.is_dirty {
            self.area = *area;
            self.buf_camera.update(ctx, &[CameraUniform::new(&viewport.camera, (*area).into())]);
            viewport.camera.is_dirty = false;
        }
        Ok(())
    }

    fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        let Rect { x, y, width, height } = self.area;
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.set_bind_group(BindGroup::Viewport.value(), &self.bind_group, &[]);
    }
}
