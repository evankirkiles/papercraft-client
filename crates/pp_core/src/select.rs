use std::collections::HashSet;

use crate::{
    id::{self, EdgeId, FaceId, Id, MeshId, VertexId},
    State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelectionActionType {
    Deselect,
    Select,
    Invert,
}

#[derive(Debug, Clone)]
pub enum SelectionActiveElement {
    Vert((id::MeshId, id::VertexId)),
    Edge((id::MeshId, id::EdgeId)),
    Face((id::MeshId, id::FaceId)),
}

#[derive(Debug, Clone)]
pub enum SelectionMode {
    Vert,
    Edge,
    Face,
}

impl From<bool> for SelectionActionType {
    fn from(value: bool) -> Self {
        if value {
            SelectionActionType::Select
        } else {
            SelectionActionType::Deselect
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub mode: SelectionMode,
    pub active_element: Option<SelectionActiveElement>,
    pub verts: HashSet<(id::MeshId, id::VertexId)>,
    pub edges: HashSet<(id::MeshId, id::EdgeId)>,
    pub faces: HashSet<(id::MeshId, id::FaceId)>,
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

impl State {
    /// Select all elements across all edeges
    pub fn select_all(&mut self) {
        self.meshes.values().for_each(|mesh| {
            (mesh.verts.indices().for_each(|id| {
                self.selection.verts.insert((mesh.id, VertexId::from_usize(id)));
            }));
            (mesh.edges.indices().for_each(|id| {
                self.selection.edges.insert((mesh.id, EdgeId::from_usize(id)));
            }));
            (mesh.faces.indices().for_each(|id| {
                self.selection.faces.insert((mesh.id, FaceId::from_usize(id)));
            }));
        });
        self.selection.is_dirty = true
    }

    /// Deselect all elements across all meshes
    pub fn deselect_all(&mut self) {
        self.selection.active_element = None;
        self.selection.verts.clear();
        self.selection.faces.clear();
        self.selection.edges.clear();
        self.selection.is_dirty = true
    }

    /// Sets the selection state of multiple vertices at once
    pub fn select_verts(&mut self, verts: &[(MeshId, id::VertexId)], action: SelectionActionType) {
        verts.iter().for_each(|id| self.select_vert(id, action, false));
        self.selection.is_dirty = true;
    }

    /// Sets the selection state of a single vertex, selecting any connected edges
    /// and faces who now have all of their elements selected.
    pub fn select_vert(
        &mut self,
        id: &(MeshId, VertexId),
        action: SelectionActionType,
        activate: bool,
    ) {
        let selected = match action {
            SelectionActionType::Deselect => false,
            SelectionActionType::Select => true,
            SelectionActionType::Invert => !self.selection.verts.contains(id),
        };
        if selected {
            self.selection.verts.insert(*id);
        } else {
            self.selection.verts.remove(id);
        }
        if activate {
            self.selection.active_element = selected.then_some(SelectionActiveElement::Vert(*id))
        }

        let (m_id, v_id) = *id;
        let mesh = &self.meshes[&m_id];
        let Some(e) = mesh[v_id].e else { return };
        let e_ids: Vec<_> = mesh
            .disk_edge_walk(e, v_id)
            .filter(|e_id| {
                if selected == self.selection.edges.contains(&(mesh.id, *e_id)) {
                    return false;
                }
                let [v1, v2] = mesh[*e_id].v;
                !selected
                    || (v1 == v_id && self.selection.verts.contains(&(mesh.id, v2)))
                    || (v2 == v_id && self.selection.verts.contains(&(mesh.id, v1)))
            })
            .collect();
        e_ids.iter().for_each(|e_id| self.select_edge(&(m_id, *e_id), selected.into()));
        self.selection.is_dirty = true
    }

    /// Sets the selection state of a single edge, selecting any connected faces
    /// who now have all edges selected.
    pub fn select_edge(&mut self, id: &(MeshId, EdgeId), action: SelectionActionType) {
        let selected = match action {
            SelectionActionType::Deselect => false,
            SelectionActionType::Select => true,
            SelectionActionType::Invert => !self.selection.edges.contains(id),
        };
        if selected {
            self.selection.edges.insert(*id);
        } else {
            self.selection.edges.remove(id);
        }
        let (m_id, e_id) = *id;
        let mesh = &self.meshes[&m_id];
        let Some(walker) = mesh.radial_loop_walk(e_id) else { return };
        walker.for_each(|l| {
            let f_id = mesh[l].f;
            let face_selected = self.selection.faces.contains(&(mesh.id, f_id));
            if selected == face_selected {
                return;
            };
            if mesh
                .face_loop_walk(f_id)
                .all(|l| self.selection.edges.contains(&(mesh.id, mesh[l].e)))
            {
                if !face_selected {
                    self.selection.faces.insert((mesh.id, f_id));
                }
            } else if face_selected {
                self.selection.faces.remove(&(mesh.id, f_id));
            }
        });
        self.selection.is_dirty = true
    }
}
