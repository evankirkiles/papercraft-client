use std::io::{BufReader, Cursor};

use material::MaterialSet;

pub mod material;
pub mod mesh;
pub mod texture;

pub use mesh::Mesh;

pub struct Model<'model, 'material> {
    pub meshes: Vec<mesh::Mesh<'model>>,
    pub materials: material::MaterialSet<'material>,
}
