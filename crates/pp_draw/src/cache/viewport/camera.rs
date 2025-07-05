use std::mem;

use pp_editor::{measures::Rect, viewport::camera::Camera};

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
pub struct CameraBindGroup {
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl CameraBindGroup {
    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(ctx, "camera".to_string(), mem::size_of::<CameraUniform>());
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera"),
                layout: &ctx.shared.bind_group_layouts.camera,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, camera: &mut impl Camera, area: &Rect<f32>) {
        if camera.is_dirty() {
            self.buf.update(ctx, &[CameraUniform::new(camera, *area)]);
            camera.set_dirty(false);
        }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[CameraUniform::bind_group_layout_entry(0)],
        })
    }
}
