use std::mem;

use pp_core::print::{Page, PrintLayout};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

/// Defines rendering resources for "pages", the surfaces where pieces are placed.
#[derive(Debug)]
pub struct PrintLayoutGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
    /// Page-specific information
    pub pages: gpu::VertBuf,
}

impl PrintLayoutGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("print_layout"),
            entries: &[PrintLayoutUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "print_layout".to_string(),
            mem::size_of::<PrintLayoutUniform>(),
        );
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("print_layout"),
                layout: &ctx.shared.bind_group_layouts.print_layout,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            pages: gpu::VertBuf::new("print_layout.pages".to_string()),
        }
    }

    pub fn prepare(&mut self, ctx: &gpu::Context, other: &mut PrintLayout) {
        // Uniform needs re-allocation
        if other.is_dirty {
            self.buf.update(ctx, &[PrintLayoutUniform::from(other)]);
            other.is_dirty = false;
        }
        // Page VBO needs re-allocation
        if other.elem_dirty {
            let pages: Vec<PageVertexAttributes> =
                other.pages.values().map(|page| page.into()).collect();
            self.pages.update(ctx, pages.as_slice());
            other.elem_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::PrintLayout.value(), &self.bind_group, &[]);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrintLayoutUniform {
    page_margin_start: [f32; 2],
    page_margin_end: [f32; 2],
    page_dimensions: [f32; 2],
    padding: [f32; 2],
}

impl PrintLayoutUniform {
    fn from(value: &PrintLayout) -> Self {
        Self {
            page_dimensions: value.page_size.dimensions().into(),
            page_margin_start: value.page_margin_start.into(),
            page_margin_end: value.page_margin_end.into(),
            padding: [0.0, 0.0],
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PageVertexAttributes {
    pos: [f32; 2],
}

impl From<&Page> for PageVertexAttributes {
    fn from(value: &Page) -> Self {
        Self { pos: value.pos.into() }
    }
}

impl PageVertexAttributes {
    pub fn vertex_attributes(base_shader_location: u32) -> [wgpu::VertexAttribute; 1] {
        [wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: base_shader_location,
        }]
    }
}
