use crate::id;

pub mod image;
pub mod texture;

#[derive(Debug)]
pub struct Material {
    pub label: String,

    pub base_color_texture: id::TextureId,
    pub base_color_factor: [f32; 4],

    pub is_dirty: bool,
}
