/// A thin wrapper around a wgpu `Texture` allowing in-place resizing.
#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new<'a>(ctx: &super::Context, descriptor: wgpu::TextureDescriptor<'a>) -> Self {
        let texture = ctx.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}
