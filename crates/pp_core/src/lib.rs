use std::collections::HashMap;

pub mod commands;
pub mod cut;
pub mod id;
pub mod material;
pub mod measures;
pub mod mesh;
pub mod select;
pub mod settings;
pub mod tool;
pub mod viewport_2d;
pub mod viewport_3d;

pub use commands::*;
pub use measures::*;

/// Represents the entire state of the "core" editor. Note that this closely
/// mimics the structure of a GLTF file.
#[derive(Default, Debug)]
pub struct State {
    pub meshes: HashMap<id::MeshId, mesh::Mesh>,
    pub materials: HashMap<id::MaterialId, material::Material>,
    pub textures: HashMap<id::TextureId, material::texture::Texture>,
    pub images: HashMap<id::ImageId, material::image::Image>,
    pub selection: select::SelectionState,
    pub viewport_3d: viewport_3d::Viewport3D,
    pub viewport_2d: viewport_2d::Viewport2D,
    pub settings: settings::Settings,
}

impl State {
    pub fn with_cube() -> Self {
        let mut state = Self::default();
        let cube = mesh::Mesh::new_cube(0);
        state.meshes.insert(cube.id, cube);
        state
    }
}
