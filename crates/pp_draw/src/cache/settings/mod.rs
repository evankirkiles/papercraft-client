use pp_editor::preferences::{theme::Theme, Preferences};
pub use theme::ThemeOverrides;
use theme::ThemeUniform;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

pub mod theme;

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
                label: Some("settings"),
                layout: &ctx.shared.bind_group_layouts.settings,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
        }
    }

    /// A settings binding fixed to `theme` under `overrides`, for passes that
    /// don't render to the screen and so never track the user's preferences.
    pub fn new_with_overrides(
        ctx: &gpu::Context,
        theme: &Theme,
        overrides: &ThemeOverrides,
    ) -> Self {
        let mut settings = Self::new(ctx);
        settings.buf.update(ctx, &[ThemeUniform::new(theme, overrides)]);
        settings
    }

    pub fn prepare(&mut self, ctx: &gpu::Context, source: &mut Preferences) {
        if source.is_dirty {
            self.buf.update(ctx, &[ThemeUniform::new(&source.theme, &ThemeOverrides::default())]);
            source.is_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Settings.value(), &self.bind_group, &[]);
    }
}
