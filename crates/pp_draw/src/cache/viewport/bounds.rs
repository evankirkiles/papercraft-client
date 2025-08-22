use std::mem;

use pp_core::measures::Rect;
use pp_editor::viewport::{Viewport, ViewportBounds};

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
pub struct ViewportBoundsGPU {
    pub area: Rect<f32>,
    pub buf: gpu::UniformBuf,
}

impl ViewportBoundsGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            area: Rect::default(),
            buf: gpu::UniformBuf::new(
                ctx,
                "viewport_bounds".to_string(),
                mem::size_of::<ViewportBoundsUniform>(),
            ),
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
