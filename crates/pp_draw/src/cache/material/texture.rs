use pp_core::{material::texture::Texture, ImageId, SamplerId};

#[derive(Clone, Debug)]
pub struct TextureGPU {
    pub image: ImageId,
    pub sampler: SamplerId,
}

impl TextureGPU {
    pub fn new(texture: &Texture) -> Self {
        Self { image: texture.image, sampler: texture.sampler }
    }
}
