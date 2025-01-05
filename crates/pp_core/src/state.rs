use std::collections::HashMap;

use crate::id::{self, Id};
use crate::material;
use crate::mesh;
use crate::viewport;

/// Represents the entire state of the "core" editor.
#[derive(Debug)]
pub struct State {
    pub meshes: HashMap<id::MeshId, mesh::Mesh>,
    pub materials: HashMap<id::MaterialId, material::Material>,
    pub viewport_3d: viewport::Viewport3D,
    pub viewport_2d: viewport::Viewport2D,
    pub viewport_split: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            meshes: Default::default(),
            materials: Default::default(),
            viewport_3d: Default::default(),
            viewport_2d: Default::default(),
            viewport_split: 0.5,
        }
    }
}
