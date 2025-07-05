use std::mem;

use pp_editor::tool::SelectBoxTool;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SelectBoxUniform {
    pub start_pos: [f32; 2],
    pub end_pos: [f32; 2],
}

impl SelectBoxUniform {
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
pub struct SelectBoxToolGPU {
    start_pos: cgmath::Point2<f32>,
    end_pos: cgmath::Point2<f32>,
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl SelectBoxToolGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tool.select_box"),
            entries: &[SelectBoxUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context, tool: &SelectBoxTool) -> Self {
        let buf =
            gpu::UniformBuf::new(ctx, "select_box".to_string(), mem::size_of::<SelectBoxUniform>());
        Self {
            start_pos: tool.start_pos,
            end_pos: tool.end_pos,
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("viewport.cutting"),
                layout: &ctx.shared.bind_group_layouts.tool.select_box,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, tool: &SelectBoxTool) {
        if self.start_pos != tool.start_pos || self.end_pos != tool.end_pos {
            self.start_pos = tool.start_pos;
            self.end_pos = tool.end_pos;
            self.buf.update(
                ctx,
                &[SelectBoxUniform {
                    start_pos: self.start_pos.into(),
                    end_pos: self.end_pos.into(),
                }],
            );
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Tool.value(), &self.bind_group, &[]);
    }
}
