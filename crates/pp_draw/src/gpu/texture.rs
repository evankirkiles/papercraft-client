/// A thin wrapper around a wgpu `Texture` allowing in-place resizing.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    /// The underlying texture descriptor used to re-create the texture at different sizes
    descriptor: wgpu::TextureDescriptor<'static>,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(ctx: &super::Context, descriptor: wgpu::TextureDescriptor<'static>) -> Self {
        let texture = ctx.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        Self { texture, view, sampler, descriptor }
    }

    /// Recreates the underlying GPU texture with the new dimensions
    pub fn resize(&mut self, ctx: &super::Context, width: u32, height: u32) {
        self.descriptor.size.width = width;
        self.descriptor.size.height = height;
        self.texture = ctx.device.create_texture(&self.descriptor);
        self.view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
    }
}
