use std::mem;

use pp_editor::{
    measures::Rect,
    viewport::{Viewport, ViewportBounds},
};

use crate::gpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportBoundsUniform {
    position: [f32; 2],
    dimensions: [f32; 2],
}

impl ViewportBoundsUniform {
    pub fn new(bounds: &ViewportBounds) -> Self {
        Self {
            position: [bounds.area.x, bounds.area.y],
            dimensions: [bounds.area.width, bounds.area.height],
        }
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
pub struct ViewportBoundsBindGroup {
    pub area: Rect<f32>,
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl ViewportBoundsBindGroup {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport"),
            entries: &[ViewportBoundsUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "viewport".to_string(),
            mem::size_of::<ViewportBoundsUniform>(),
        );
        Self {
            area: Rect::default(),
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("viewport"),
                layout: &ctx.shared.bind_group_layouts.viewport,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, viewport: &mut Viewport) {
        if !viewport.bounds.is_dirty {
            return;
        };
        self.area = viewport.bounds.area;
        self.buf.update(ctx, &[ViewportBoundsUniform::new(&viewport.bounds)]);
        viewport.bounds.is_dirty = false;
    }
}
