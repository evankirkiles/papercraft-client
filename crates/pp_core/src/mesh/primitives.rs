use crate::id::{self, Id};

use super::Mesh;

impl Mesh {
    pub fn new_tri(id: u32) -> Self {
        let mut mesh = Self::new(id::MeshId::new(id), "CUBE".to_string());
        let v = &[
            mesh.add_vertex([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            mesh.add_vertex([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            mesh.add_vertex([1.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
        ];
        mesh.add_face(&[v[0], v[1], v[2]]);
        mesh
    }

    pub fn new_cube(id: u32) -> Self {
        let mut mesh = Self::new(id::MeshId::new(id), "CUBE".to_string());
        let v = &[
            mesh.add_vertex([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0]), // Bottom-left-front
            mesh.add_vertex([-0.5, 0.5, 0.0], [1.0, 0.0, 0.0]),  // Bottom-right-front
            mesh.add_vertex([0.5, 0.5, 0.0], [1.0, 0.0, 0.0]),   // Bottom-right-back
            mesh.add_vertex([0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),  // Bottom-left-back
            mesh.add_vertex([-0.5, -0.5, 1.0], [0.0, 0.0, 1.0]), // Front-left-front
            mesh.add_vertex([-0.5, 0.5, 1.0], [0.0, 0.0, 1.0]),  // Front-right-front
            mesh.add_vertex([0.5, 0.5, 1.0], [0.0, 0.0, 1.0]),   // Front-right-back
            mesh.add_vertex([0.5, -0.5, 1.0], [0.0, 0.0, 1.0]),  // Front-left-back
        ];
        // Bottom
        mesh.add_face(&[v[0], v[1], v[2]]);
        mesh.add_face(&[v[0], v[2], v[3]]);
        // Top
        mesh.add_face(&[v[4], v[7], v[6]]);
        mesh.add_face(&[v[4], v[6], v[5]]);
        // Front
        mesh.add_face(&[v[0], v[4], v[5]]);
        mesh.add_face(&[v[0], v[5], v[1]]);
        // Back
        mesh.add_face(&[v[3], v[2], v[6]]);
        mesh.add_face(&[v[3], v[6], v[7]]);
        // Left
        mesh.add_face(&[v[0], v[3], v[7]]);
        mesh.add_face(&[v[0], v[7], v[4]]);
        // Right
        mesh.add_face(&[v[1], v[5], v[6]]);
        mesh.add_face(&[v[1], v[6], v[2]]);
        mesh
    }
}
