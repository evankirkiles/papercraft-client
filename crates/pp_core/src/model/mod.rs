use std::io::{BufReader, Cursor};

use material::MaterialSet;

pub mod material;
pub mod mesh;
pub mod texture;

pub struct Model<'model, 'material> {
    pub meshes: mesh::Mesh<'model>,
    pub materials: material::MaterialSet<'material>,
}
