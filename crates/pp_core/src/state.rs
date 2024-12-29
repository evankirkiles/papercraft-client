use std::collections::HashMap;

use crate::id::{self, Id};
use crate::material;
use crate::mesh;
use crate::viewport;

/// Represents the entire state of the "core" editor.
pub struct State {
    pub meshes: HashMap<id::MeshId, mesh::Mesh>,
    pub materials: HashMap<id::MaterialId, material::Material>,
    pub viewports: HashMap<id::ViewportId, viewport::Viewport>,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            meshes: Default::default(),
            materials: Default::default(),
            viewports: Default::default(),
        };
        state.viewports.insert(id::ViewportId::new(0), viewport::Viewport::default());
        state
    }
}
