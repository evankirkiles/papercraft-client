pub struct FrameBuffer {
    // If true, indicates that the first color attachment for this frame buffer
    // will be a view into the current swapchain texture (at render time).
    is_swapchain: bool,
    /// Additional color attachments. Required if not `for_swapchain`
    color_textures: Vec<super::Texture>,
    depth_stencil_texture: Option<super::Texture>,
}

impl FrameBuffer {
    /// Initializes a frame buffer whose render call writes to the swapchain
    /// and then presents it to the screen.
    pub fn from_swapchain(ctx: &super::Context) -> Self {
        Self {
            is_swapchain: true,
            color_textures: vec![], // Surface will provide the color texture
            depth_stencil_texture: Some(super::Texture::create_depth_texture(ctx)),
        }
    }

    /// Creates a render pass to draw into the FrameBuffer. If this frame buffer
    /// is a Presentation frame buffer, the first color attachment used will be
    /// the current surface texture. The surface texture is presented on completion.
    pub fn render<F>(&self, ctx: &super::Context, render_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(&mut wgpu::RenderPass),
    {
        let mut encoder =
            ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // If a presentation frame buffer, use the current surface texture as
        // the first color attachment.
        let (output, view) = if self.is_swapchain {
            let output = ctx.surface.get_current_texture().unwrap();
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(output), Some(view))
        } else {
            (None, None)
        };

        {
            let color_attachments = [view.as_ref()]
                .into_iter()
                .chain(self.color_textures.iter().map(|tex| Some(&tex.view)))
                .map(|view| {
                    view.map(|view| wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.05,
                                g: 0.05,
                                b: 0.05,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })
                })
                .collect::<Vec<_>>();

            let depth_stencil_attachment = self.depth_stencil_texture.as_ref().map(|texture| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: &texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: color_attachments.as_slice(),
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Call the render closure
            render_fn(&mut render_pass);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        if let Some(output) = output {
            output.present();
        }
        Ok(())
    }

    /// Recreates all attachments with the proper size
    pub fn resize(&mut self, ctx: &super::Context) {
        self.depth_stencil_texture = Some(super::Texture::create_depth_texture(ctx))
    }
}
