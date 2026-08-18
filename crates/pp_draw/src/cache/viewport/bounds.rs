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
        Self::for_area(bounds.area)
    }

    /// The uniform for a bare area, for passes with no editor viewport behind
    /// them - the print pass renders into a texture that is all "viewport".
    pub fn for_area(area: Rect<f32>) -> Self {
        Self { position: [area.x, area.y], dimensions: [area.width, area.height] }
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
