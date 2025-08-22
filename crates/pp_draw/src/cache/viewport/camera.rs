use std::mem;

use pp_core::measures::Rect;
use pp_editor::viewport::{camera::Camera, ViewportBounds};

use crate::gpu::{self};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
}

impl CameraUniform {
    pub fn new(camera: &impl Camera, area: Rect<f32>) -> Self {
        Self { view_proj: camera.view_proj(area.into()).into(), eye: camera.eye() }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraGPU {
    pub buf: gpu::UniformBuf,
}

impl CameraGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            buf: gpu::UniformBuf::new(ctx, "camera".to_string(), mem::size_of::<CameraUniform>()),
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, camera: &mut impl Camera, bounds: &ViewportBounds) {
        if bounds.is_dirty || camera.is_dirty() {
            self.buf.update(ctx, &[CameraUniform::new(camera, bounds.area)]);
            camera.set_dirty(false);
        }
    }
}
