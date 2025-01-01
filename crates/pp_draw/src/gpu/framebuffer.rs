use super::Texture;

pub struct FrameBuffer {
    pub label: String,
    /// Additional color attachments. Required if not `for_swapchain`
    pub color_textures: Vec<super::Texture>,
    pub depth_stencil_texture: Option<super::Texture>,
}

impl FrameBuffer {
    pub fn new_presentation(ctx: &super::Context, width: u32, height: u32) -> Self {
        Self {
            label: "presentation".into(),
            color_textures: Vec::from([Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("presentation.color"),
                    mip_level_count: 1,
                    sample_count: (&ctx.settings.msaa_level).into(),
                    dimension: wgpu::TextureDimension::D2,
                    format: ctx.config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                },
                true,
            )]),
            depth_stencil_texture: Some(Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("presentation.depth_stencil"),
                    mip_level_count: 1,
                    sample_count: (&ctx.settings.msaa_level).into(),
                    dimension: wgpu::TextureDimension::D2,
                    format: Texture::DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                },
                false,
            )),
        }
    }

    /// Creates a render pass and draws into the attachments, submitting once
    /// `render_fn` completes.
    pub fn render<F>(
        &self,
        ctx: &super::Context,
        external_resolve_target: Option<&wgpu::TextureView>,
        render_fn: F,
    ) where
        F: FnOnce(&mut wgpu::RenderPass),
    {
        let mut encoder =
            ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: self
                    .color_textures
                    .iter()
                    .map(|tex| Some(tex.color_attachment(external_resolve_target)))
                    .collect::<Vec<_>>()
                    .as_slice(),
                depth_stencil_attachment: self
                    .depth_stencil_texture
                    .as_ref()
                    .map(|tex| tex.depth_stencil_attachment()),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Call the render closure
            render_fn(&mut render_pass);
        }
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Recreates all attachments with the proper size
    pub fn resize(&mut self, ctx: &super::Context, width: u32, height: u32) {
        self.color_textures.iter_mut().for_each(|tex| tex.resize(ctx, width, height));
        if let Some(tex) = self.depth_stencil_texture.as_mut() {
            tex.resize(ctx, width, height);
        }
    }
}
