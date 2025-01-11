use std::collections::HashSet;

use crate::{id, mesh::Mesh};

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub verts: HashSet<(id::MeshId, id::VertexId)>,
    pub edges: HashSet<(id::MeshId, id::EdgeId)>,
    pub faces: HashSet<(id::MeshId, id::FaceId)>,
    pub is_dirty: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            verts: Default::default(),
            edges: Default::default(),
            faces: Default::default(),
            is_dirty: true,
        }
    }
}

impl SelectionState {
    pub fn deselect_all(&mut self) {
        self.verts.clear();
        self.edges.clear();
        self.faces.clear();
        self.is_dirty = true
    }

    pub fn select_verts(&mut self, mesh: &Mesh, verts: &[id::VertexId]) {
        verts.iter().for_each(|v| {
            self.verts.insert((mesh.id, *v));
        });
        self.is_dirty = true
    }

    fn ensure_valid(&mut self) {}
}
