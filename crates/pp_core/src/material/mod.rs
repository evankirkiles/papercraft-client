use crate::id;

/// A Material is a collection of different textures applied to tris.
pub struct Material {
    pub id: id::MaterialId,
    pub label: String,
}