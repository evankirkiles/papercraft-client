use std::collections::HashSet;

use crate::id::{self, Id};
use crate::State;

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
    pub fn select_all(&mut self, action: SelectionActionType) {
        match action {
            SelectionActionType::Deselect => {
                self.selection.verts.clear();
                self.selection.faces.clear();
                self.selection.edges.clear();
            }
            SelectionActionType::Select => {
                self.meshes.values().for_each(|mesh| {
                    (mesh.verts.indices().for_each(|id| {
                        self.selection.verts.insert((mesh.id, id::VertexId::from_usize(id)));
                    }));
                    (mesh.edges.indices().for_each(|id| {
                        self.selection.edges.insert((mesh.id, id::EdgeId::from_usize(id)));
                    }));
                    (mesh.faces.indices().for_each(|id| {
                        self.selection.faces.insert((mesh.id, id::FaceId::from_usize(id)));
                    }));
                });
            }
            SelectionActionType::Invert => todo!(),
        };
        self.selection.active_element = None;
        self.selection.is_dirty = true
    }

    /// Sets the selection state of a single vertex, selecting any connected edges
    /// and faces who now have all of their elements selected.
    pub fn select_vert(
        &mut self,
        id: &(id::MeshId, id::VertexId),
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
            .iter_vert_edges(e, v_id)
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
        e_ids.iter().for_each(|e_id| self.select_edge(&(m_id, *e_id), selected.into(), false));
        self.selection.is_dirty = true
    }

    /// Sets the selection state of a single edge, selecting any connected faces
    /// who now have all edges selected.
    pub fn select_edge(
        &mut self,
        id: &(id::MeshId, id::EdgeId),
        action: SelectionActionType,
        activate: bool,
    ) {
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
        if activate {
            self.selection.active_element = selected.then_some(SelectionActiveElement::Edge(*id))
        }
        self.selection.is_dirty = true;
        let (m_id, e_id) = *id;
        let mesh = &self.meshes[&m_id];
        if let Some(walker) = mesh.iter_edge_loops(e_id) {
            walker.for_each(|l| {
                let f_id = mesh[l].f;
                let face_selected = self.selection.faces.contains(&(mesh.id, f_id));
                if selected == face_selected {
                    return;
                };
                if mesh
                    .iter_face_loops(f_id)
                    .all(|l| self.selection.edges.contains(&(mesh.id, mesh[l].e)))
                {
                    if !face_selected {
                        self.selection.faces.insert((mesh.id, f_id));
                    }
                } else if face_selected {
                    self.selection.faces.remove(&(mesh.id, f_id));
                }
            })
        }
    }

    /// Sets the selection state of a single face.
    pub fn select_face(
        &mut self,
        id: &(id::MeshId, id::FaceId),
        action: SelectionActionType,
        activate: bool,
    ) {
        let selected = match action {
            SelectionActionType::Deselect => false,
            SelectionActionType::Select => true,
            SelectionActionType::Invert => !self.selection.faces.contains(id),
        };
        if selected {
            self.selection.faces.insert(*id);
        } else {
            self.selection.faces.remove(id);
        }
        if activate {
            self.selection.active_element = selected.then_some(SelectionActiveElement::Face(*id))
        }
        self.selection.is_dirty = true
    }
}
