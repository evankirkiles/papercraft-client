/// A MaterialSet is a collection of materials all used to render an object
/// Faces refer to a specific material by index in the mats property
pub struct MaterialSet<'a> {
    pub label: Option<&'a str>,
    pub mats: Vec<Material<'a>>,
}

/// A Material is a collection of different textures applied to tris.
pub struct Material<'a> {
    pub label: Option<&'a str>,
    pub diffuse_texture: Texture<'a>,
}

/// A Texture is raw bytes making up an image.
pub struct Texture<'a> {
    pub label: Option<&'a str>,
    pub image: image::DynamicImage,
}

impl MaterialSet<'_> {
    pub fn new() -> Self {
        Self {
            label: Some("HI"),
            mats: vec![],
        }
    }
}
