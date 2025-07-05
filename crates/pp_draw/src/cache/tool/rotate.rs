use pp_editor::tool::RotateTool;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RotateToolUniform {
    pub center_pos: [f32; 2],
    pub curr_pos: [f32; 2],
}

impl RotateToolUniform {
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
pub struct RotateToolGPU {
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl RotateToolGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tool.rotate"),
            entries: &[RotateToolUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context, _: &RotateTool) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "tool.rotate".to_string(),
            std::mem::size_of::<RotateToolUniform>(),
        );
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tool.rotate"),
                layout: &ctx.shared.bind_group_layouts.tool.rotate,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, tool: &mut RotateTool) {
        if tool.is_dirty {
            let uniform = RotateToolUniform {
                center_pos: tool.center_pos.into(),
                curr_pos: tool.curr_pos.unwrap_or(tool.center_pos).into(),
            };
            self.buf.update(ctx, &[uniform]);
            tool.is_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Tool.value(), &self.bind_group, &[]);
    }
}
