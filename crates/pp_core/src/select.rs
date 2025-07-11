use std::collections::HashSet;

use itertools::Itertools;

use crate::id::{self, Id};
use crate::{MeshId, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelectionActionType {
    Deselect,
    Select,
    Invert,
}

#[derive(Debug, Clone, Copy)]
pub enum SelectionActiveElement {
    Vert((MeshId, id::VertexId)),
    Edge((MeshId, id::EdgeId)),
    Face((MeshId, id::FaceId)),
}

#[derive(Default, Debug, Clone)]
pub enum SelectionMode {
    #[default]
    Vert,
    Edge,
    Face,
    Piece,
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

#[derive(Default, Debug, Clone)]
pub struct SelectionState {
    pub active_element: Option<SelectionActiveElement>,
    pub verts: HashSet<(MeshId, id::VertexId)>,
    pub edges: HashSet<(MeshId, id::EdgeId)>,
    pub faces: HashSet<(MeshId, id::FaceId)>,
    pub pieces: HashSet<(MeshId, id::PieceId)>,
    pub is_dirty: bool,
}

impl State {
    /// Select all elements
    pub fn select_all(&mut self, action: SelectionActionType) {
        match action {
            SelectionActionType::Deselect => {
                self.selection.verts.clear();
                self.selection.faces.clear();
                self.selection.edges.clear();
                self.selection.pieces.clear();
            }
            SelectionActionType::Select => {
                self.meshes.iter().for_each(|(m_id, mesh)| {
                    (mesh.verts.indices().for_each(|id| {
                        self.selection.verts.insert((m_id, id::VertexId::from_usize(id)));
                    }));
                    (mesh.edges.indices().for_each(|id| {
                        self.selection.edges.insert((m_id, id::EdgeId::from_usize(id)));
                    }));
                    (mesh.faces.indices().for_each(|id| {
                        self.selection.faces.insert((m_id, id::FaceId::from_usize(id)));
                    }));
                    (mesh.pieces.indices().for_each(|id| {
                        self.selection.pieces.insert((m_id, id::PieceId::from_usize(id)));
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
        id: &(MeshId, id::VertexId),
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
        let mesh = &self.meshes[m_id];
        let Some(e) = mesh[v_id].e else { return };
        let e_ids: Vec<_> = mesh
            .iter_vert_edges(e, v_id)
            .filter(|e_id| {
                if selected == self.selection.edges.contains(&(m_id, *e_id)) {
                    return false;
                }
                let [v1, v2] = mesh[*e_id].v;
                !selected
                    || (v1 == v_id && self.selection.verts.contains(&(m_id, v2)))
                    || (v2 == v_id && self.selection.verts.contains(&(m_id, v1)))
            })
            .collect();
        e_ids
            .iter()
            .for_each(|e_id| self.select_edge(&(m_id, *e_id), selected.into(), false, true));
        self.selection.is_dirty = true
    }

    /// Sets the selection state of a single edge, selecting any connected faces
    /// who now have all edges selected.
    pub fn select_edge(
        &mut self,
        id: &(MeshId, id::EdgeId),
        action: SelectionActionType,
        activate: bool,
        include_faces: bool,
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

        // Propagate selection to faces
        if include_faces {
            let (m_id, e_id) = *id;
            let mesh = &self.meshes[m_id];
            let select_mode =
                if selected { SelectionActionType::Select } else { SelectionActionType::Deselect };
            let updated_faces: Option<Vec<_>> = mesh.iter_edge_loops(e_id).map(|walker| {
                walker
                    .filter_map(|l| {
                        let f_id = mesh[l].f;
                        let face_selected = self.selection.faces.contains(&(m_id, f_id));
                        if selected == face_selected {
                            return None;
                        };
                        if mesh
                            .iter_face_loops(f_id)
                            .all(|l| self.selection.edges.contains(&(m_id, mesh[l].e)))
                        {
                            if !face_selected {
                                return Some((m_id, f_id));
                            }
                        } else if face_selected {
                            return Some((m_id, f_id));
                        };
                        None
                    })
                    .collect()
            });
            if let Some(updated_faces) = updated_faces {
                updated_faces.iter().for_each(|id| self.select_face(id, select_mode, false, false));
            }
        }
    }

    /// Sets the selection state of a single face.
    pub fn select_face(
        &mut self,
        id: &(MeshId, id::FaceId),
        action: SelectionActionType,
        activate: bool,
        include_edges: bool,
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

        // Propagate selection to edges
        if include_edges {
            let (m_id, f_id) = *id;
            let mesh = &self.meshes[m_id];
            let select_mode =
                if selected { SelectionActionType::Select } else { SelectionActionType::Deselect };
            let updated_edges: Vec<_> = mesh
                .iter_face_loops(f_id)
                .filter_map(|l| {
                    let e_id = mesh[l].e;
                    let edge_selected = self.selection.edges.contains(&(m_id, e_id));
                    if selected != edge_selected {
                        Some((m_id, e_id))
                    } else {
                        None
                    }
                })
                .collect();
            updated_edges.iter().for_each(|id| self.select_edge(id, select_mode, false, false));
        }
        self.selection.is_dirty = true
    }

    /// Selects all the faces / edges / verts within a piece
    pub fn select_piece(&mut self, id: &(MeshId, id::PieceId), action: SelectionActionType) {
        let selected = match action {
            SelectionActionType::Deselect => false,
            SelectionActionType::Select => true,
            // TODO: Only "Deselect" if all faces are selected
            SelectionActionType::Invert => !self.selection.pieces.contains(id),
        };

        // Propagate selection to faces
        let (m_id, p_id) = *id;
        let mesh = &self.meshes[m_id];
        let select_mode =
            if selected { SelectionActionType::Select } else { SelectionActionType::Deselect };
        let updated_faces: Vec<_> = mesh
            .iter_connected_faces(mesh[p_id].f)
            .filter_map(|f_id| {
                let face_selected = self.selection.faces.contains(&(m_id, f_id));
                (selected != face_selected).then_some((m_id, f_id))
            })
            .collect();
        updated_faces.iter().for_each(|id| self.select_face(id, select_mode, false, true));

        self.selection.is_dirty = true
    }

    /// Returns all the pieces which have at least one face selected in the mesh
    pub fn get_selected_pieces(&self) -> Vec<(MeshId, id::PieceId)> {
        self.selection
            .faces
            .iter()
            .filter_map(|(m_id, f_id)| self.meshes[*m_id][*f_id].p.map(|p_id| (*m_id, p_id)))
            .unique()
            .collect()
    }
}
