use crate::TextureId;

pub mod image;
pub mod texture;

pub use image::*;
pub use texture::*;

#[derive(Debug)]
pub struct Material {
    pub label: String,

    pub base_color_texture: TextureId,
    pub base_color_factor: [f32; 4],

    pub is_dirty: bool,
}
