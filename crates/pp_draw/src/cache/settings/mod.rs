use pp_editor::settings::Settings;
use theme::ThemeUniform;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

mod theme;

/// Defines rendering resources for "pages", the surfaces where pieces are placed.
#[derive(Debug)]
pub struct SettingsGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
}

impl SettingsGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("settings"),
            entries: &[ThemeUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(ctx, "settings".to_string(), size_of::<ThemeUniform>());
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("print_layout"),
                layout: &ctx.shared.bind_group_layouts.print_layout,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    pub fn prepare(&mut self, ctx: &gpu::Context, source: &mut Settings) {
        if source.is_dirty {
            self.buf.update(ctx, &[ThemeUniform::from(&source.theme)]);
            source.is_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Settings.value(), &self.bind_group, &[]);
    }
}
