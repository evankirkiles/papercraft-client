use std::collections::HashSet;

use crate::{id, mesh::Mesh};

#[derive(Debug, Clone)]
pub enum SelectionActiveElement {
    Vert((id::MeshId, id::VertexId)),
    Edge((id::MeshId, id::EdgeId)),
    Face((id::MeshId, id::FaceId)),
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub active_element: Option<SelectionActiveElement>,
    pub verts: HashSet<(id::MeshId, id::VertexId)>,
    pub edges: HashSet<(id::MeshId, id::EdgeId)>,
    pub faces: HashSet<(id::MeshId, id::FaceId)>,
    pub is_dirty: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            active_element: None,
            verts: Default::default(),
            edges: Default::default(),
            faces: Default::default(),
            is_dirty: true,
        }
    }
}

impl SelectionState {
    pub fn deselect_all(&mut self) {
        self.active_element = None;
        self.verts.clear();
        self.edges.clear();
        self.faces.clear();
        self.is_dirty = true
    }

    pub fn select_verts(&mut self, mesh: &Mesh, verts: &[id::VertexId]) {
        verts.iter().for_each(|v| {
            let key = (mesh.id, *v);
            self.verts.insert((mesh.id, *v));
            if verts.len() == 1 {
                self.active_element = Some(SelectionActiveElement::Vert(key));
            }
        });
        self.is_dirty = true
    }

    pub fn toggle_verts(&mut self, mesh: &Mesh, verts: &[id::VertexId]) {
        verts.iter().for_each(|v| {
            let key = (mesh.id, *v);
            if self.verts.contains(&key) {
                self.verts.remove(&key);
            } else {
                self.verts.insert(key);
                if verts.len() == 1 {
                    self.active_element = Some(SelectionActiveElement::Vert(key));
                }
            }
        });
        self.is_dirty = true
    }

    fn ensure_valid(&mut self) {}
}
