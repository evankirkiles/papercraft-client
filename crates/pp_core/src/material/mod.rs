use crate::id;

pub mod texture;

#[derive(Debug)]
pub enum MaterialDiffuse {
    Texture(id::TextureId),
    Color([f32; 4]),
}

#[derive(Debug)]
pub struct Material {
    pub id: id::MaterialId,
    pub label: String,

    /// The diffuse component of the material
    pub diffuse: MaterialDiffuse,
}
