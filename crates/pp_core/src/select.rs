use std::collections::{HashMap, HashSet};

use crate::{id, mesh::Mesh};

type MeshVertId = (id::MeshId, id::VertexId);
type MeshEdgeId = (id::MeshId, id::EdgeId);
type MeshFaceId = (id::MeshId, id::FaceId);

#[derive(Debug, Clone)]
pub enum SelectionActiveElement {
    Vert(MeshVertId),
    Edge(MeshEdgeId),
    Face(MeshFaceId),
}

#[derive(Debug, Clone)]
pub enum SelectionMode {
    Vert,
    Edge,
    Face,
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub mode: SelectionMode,
    pub active_element: Option<SelectionActiveElement>,
    pub verts: HashSet<MeshVertId>,
    pub edges: HashSet<MeshEdgeId>,
    pub faces: HashSet<MeshFaceId>,
    pub is_dirty: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Vert,
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

    pub fn set_vert(&mut self, mesh: &Mesh, v_id: id::VertexId, selected: bool) {
        if selected {
            self.verts.insert((mesh.id, v_id));
        } else {
            self.verts.remove(&(mesh.id, v_id));
        }
        let Some(e) = mesh[v_id].e else { return };
        mesh.disk_edge_walk(e, v_id).for_each(|e_id| {
            if selected == self.edges.contains(&(mesh.id, e_id)) {
                return;
            }
            let [v1, v2] = mesh[e_id].v;
            if !selected
                || (v1 == v_id && self.verts.contains(&(mesh.id, v2)))
                || (v2 == v_id && self.verts.contains(&(mesh.id, v1)))
            {
                self.set_edge(mesh, e_id, selected)
            }
        });
    }

    pub fn set_edge(&mut self, mesh: &Mesh, e_id: id::EdgeId, selected: bool) {
        if selected {
            self.edges.insert((mesh.id, e_id));
        } else {
            self.edges.remove(&(mesh.id, e_id));
        }
        let Some(walker) = mesh.radial_loop_walk(e_id) else { return };
        walker.for_each(|l| {
            let f_id = mesh[l].f;
            let face_selected = self.faces.contains(&(mesh.id, f_id));
            if selected == face_selected {
                return;
            };
            if mesh.face_loop_walk(f_id).all(|l| self.edges.contains(&(mesh.id, mesh[l].e))) {
                if !face_selected {
                    self.faces.insert((mesh.id, f_id));
                }
            } else if face_selected {
                self.faces.remove(&(mesh.id, f_id));
            }
        });
    }

    pub fn select_verts(&mut self, mesh: &Mesh, verts: &[id::VertexId]) {
        verts.iter().for_each(|v| {
            self.set_vert(mesh, *v, true);
        });
        self.active_element = if verts.len() == 1 {
            Some(SelectionActiveElement::Vert((mesh.id, verts[0])))
        } else {
            None
        };
        self.is_dirty = true
    }

    pub fn toggle_verts(&mut self, mesh: &Mesh, verts: &[id::VertexId]) {
        verts.iter().for_each(|v| {
            self.set_vert(mesh, *v, !self.verts.contains(&(mesh.id, *v)));
        });
        let key = (mesh.id, verts[0]);
        self.active_element = if verts.len() == 1 && self.verts.contains(&key) {
            Some(SelectionActiveElement::Vert(key))
        } else {
            None
        };
        self.is_dirty = true
    }
}
