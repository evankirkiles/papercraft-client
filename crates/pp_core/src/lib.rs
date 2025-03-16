use std::collections::HashMap;

use mesh::Mesh;

pub mod id;
pub mod material;
pub mod mesh;
pub mod select;
pub mod viewport;

/// Represents the entire state of the "core" editor.
#[derive(Debug)]
pub struct State {
    pub meshes: HashMap<id::MeshId, mesh::Mesh>,
    pub materials: HashMap<id::MaterialId, material::Material>,
    pub viewport_3d: viewport::Viewport3D,
    pub viewport_2d: viewport::Viewport2D,
    pub viewport_split_x: f64,
    pub viewport_split_y: f64,
    pub selection: select::SelectionState,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            meshes: Default::default(),
            materials: Default::default(),
            viewport_3d: Default::default(),
            viewport_2d: Default::default(),
            viewport_split_x: 0.5,
            viewport_split_y: 1.0,
            selection: Default::default(),
        };

        // Set up basic initial scene
        let cube = Mesh::new_cube(0);
        state.meshes.insert(cube.id, cube);

        // Now return the populated state
        state
    }
}
