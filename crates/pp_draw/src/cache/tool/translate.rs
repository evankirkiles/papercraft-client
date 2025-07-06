use bitflags::bitflags;
use pp_editor::tool::{translate::TranslateAxisLock, TranslateTool};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};
bitflags! {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct TranslateFlags: u32 {
        const X_LOCKED = 1 << 0;
        const Y_LOCKED = 1 << 1;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TranslateToolUniform {
    pub center_pos: [f32; 2],
    pub flags: u32,
    pub padding: u32,
}

impl TranslateToolUniform {
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
pub struct TranslateToolGPU {
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,
}

impl TranslateToolGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tool.translate"),
            entries: &[TranslateToolUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context, _: &TranslateTool) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "tool.translate".to_string(),
            std::mem::size_of::<TranslateToolUniform>(),
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

    pub fn sync(&mut self, ctx: &gpu::Context, tool: &mut TranslateTool) {
        if tool.is_dirty {
            let uniform = TranslateToolUniform {
                center_pos: tool.center_pos.into(),
                flags: (match tool.axis_lock {
                    Some(TranslateAxisLock::X) => TranslateFlags::X_LOCKED,
                    Some(TranslateAxisLock::Y) => TranslateFlags::Y_LOCKED,
                    None => TranslateFlags::empty(),
                })
                .bits(),
                padding: 0,
            };
            self.buf.update(ctx, &[uniform]);
            tool.is_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Tool.value(), &self.bind_group, &[]);
    }
}
