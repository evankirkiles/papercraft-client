/// A thin wrapper around a wgpu `Texture` allowing in-place resizing.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,

    /// If true, "resolves" this texture to another provided textureview
    is_ext_resolve_target: bool,

    /// The underlying texture descriptor used to re-create the texture at different sizes
    descriptor: wgpu::TextureDescriptor<'static>,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(
        ctx: &super::Context,
        descriptor: wgpu::TextureDescriptor<'static>,
        is_ext_resolve_target: bool,
    ) -> Self {
        let texture = ctx.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view, descriptor, is_ext_resolve_target }
    }

    /// Recreates the underlying GPU texture with the new dimensions
    pub fn resize(&mut self, ctx: &super::Context, width: u32, height: u32) {
        self.descriptor.size.width = width;
        self.descriptor.size.height = height;
        self.texture = ctx.device.create_texture(&self.descriptor);
        self.view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
    }

    /// Returns a RenderPassColorAttachment. If this texture was created with
    /// "resolves_to_target: true" and a resolve_target is provided, then
    /// the returned ColorPassAttachment resolves to
    pub fn color_attachment<'tex, 'a: 'tex>(
        &'a self,
        ext_resolve_target: Option<&'tex wgpu::TextureView>,
    ) -> wgpu::RenderPassColorAttachment<'tex> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: if self.is_ext_resolve_target { ext_resolve_target } else { None },
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
                store: wgpu::StoreOp::Store,
            },
        }
    }

    /// Returns this texture as a DepthStencilAttachment.
    pub fn depth_stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }

    // pub fn create_depth_texture(ctx: &super::Context) -> Self {
    //     let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
    //         label: None,
    //         size: wgpu::Extent3d {
    //             width: ctx.config.width.max(1),
    //             height: ctx.config.height.max(1),
    //             depth_or_array_layers: 1,
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: Self::DEPTH_FORMAT,
    //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    //         view_formats: &[],
    //     });
    //     let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    //     let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
    //         address_mode_u: wgpu::AddressMode::ClampToEdge,
    //         address_mode_v: wgpu::AddressMode::ClampToEdge,
    //         address_mode_w: wgpu::AddressMode::ClampToEdge,
    //         mag_filter: wgpu::FilterMode::Linear,
    //         min_filter: wgpu::FilterMode::Linear,
    //         mipmap_filter: wgpu::FilterMode::Nearest,
    //         compare: Some(wgpu::CompareFunction::LessEqual),
    //         lod_min_clamp: 0.0,
    //         lod_max_clamp: 100.0,
    //         ..Default::default()
    //     });
    //     Self { texture, view }
    // }
}
