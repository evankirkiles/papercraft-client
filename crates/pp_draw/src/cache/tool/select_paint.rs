use pp_editor::tool::SelectPaintTool;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SelectPaintToolUniform {
    pub center: [f32; 2],
    pub radius: f32,
    pub _pad: f32,
}

impl SelectPaintToolUniform {
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
pub struct SelectPaintToolGPU {
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl SelectPaintToolGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tool.select_paint"),
            entries: &[SelectPaintToolUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context, _: &SelectPaintTool) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "tool.select_paint".to_string(),
            std::mem::size_of::<SelectPaintToolUniform>(),
        );
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tool.select_paint"),
                layout: &ctx.shared.bind_group_layouts.tool.select_paint,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, tool: &mut SelectPaintTool) {
        if tool.is_dirty {
            let uniform = SelectPaintToolUniform {
                center: tool.cursor_pos.into(),
                radius: tool.radius,
                _pad: 0.0,
            };
            self.buf.update(ctx, &[uniform]);
            tool.is_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Tool.value(), &self.bind_group, &[]);
    }
}
