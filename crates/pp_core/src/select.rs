use std::collections::{BTreeMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::id::{self, Id};
use crate::{MeshId, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelectionActionType {
    Deselect,
    Select,
    Invert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SelectionActiveElement {
    Vert((MeshId, id::VertexId)),
    Edge((MeshId, id::EdgeId)),
    Face((MeshId, id::FaceId)),
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SelectionState {
    pub active_element: Option<SelectionActiveElement>,
    pub verts: HashSet<(MeshId, id::VertexId)>,
    pub edges: HashSet<(MeshId, id::EdgeId)>,
    pub faces: HashSet<(MeshId, id::FaceId)>,
    pub pieces: HashSet<(MeshId, id::FaceId)>,
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
                    (mesh.pieces.keys().for_each(|id| {
                        self.selection.pieces.insert((m_id, *id));
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
                    if selected == edge_selected {
                        return None;
                    }
                    // Don't deselect an edge that's still owned by another selected face.
                    if !selected
                        && mesh.iter_edge_loops(e_id).into_iter().flatten().any(|l| {
                            let other_f = mesh[l].f;
                            other_f != f_id && self.selection.faces.contains(&(m_id, other_f))
                        })
                    {
                        return None;
                    }
                    Some((m_id, e_id))
                })
                .collect();
            updated_edges.iter().for_each(|id| self.select_edge(id, select_mode, false, false));
        }
        self.selection.is_dirty = true
    }

    /// Selects all the faces / edges / verts within a piece
    pub fn select_piece(&mut self, id: &(MeshId, id::FaceId), action: SelectionActionType) {
        let selected = match action {
            SelectionActionType::Deselect => false,
            SelectionActionType::Select => true,
            // TODO: Only "Deselect" if all faces are selected
            SelectionActionType::Invert => !self.selection.pieces.contains(id),
        };

        // Propagate selection to faces
        let (m_id, f_id) = *id;
        let mesh = &self.meshes[m_id];
        let select_mode =
            if selected { SelectionActionType::Select } else { SelectionActionType::Deselect };
        let updated_faces: Vec<_> = mesh
            .iter_connected_faces(f_id)
            .filter_map(|f_id| {
                let face_selected = self.selection.faces.contains(&(m_id, f_id));
                (selected != face_selected).then_some((m_id, f_id))
            })
            .collect();
        updated_faces.iter().for_each(|id| self.select_face(id, select_mode, false, true));

        self.selection.is_dirty = true
    }

    /// Replaces the selection with the ring of edges one step outside it: every
    /// uncut edge whose two verts are both neighbors of the current selection's
    /// boundary verts, without lying on that boundary themselves. Repeatedly
    /// expanding marches a cut outwards across a mesh; edges which are already
    /// cut are skipped, since there's nothing left to cut on them.
    ///
    /// Returns whether the selection actually changed.
    pub fn expand_selection(&mut self) -> bool {
        // Group by mesh so each mesh expands independently, and so the walk
        // doesn't depend on the hash order of `selection.edges`.
        let mut by_mesh: BTreeMap<MeshId, Vec<id::EdgeId>> = BTreeMap::new();
        self.selection.edges.iter().for_each(|(m_id, e_id)| {
            by_mesh.entry(*m_id).or_default().push(*e_id);
        });

        let expanded: HashSet<(MeshId, id::EdgeId)> = by_mesh
            .iter()
            .flat_map(|(m_id, e_ids)| {
                let mesh = &self.meshes[*m_id];
                // The boundary: every vert touched by a selected edge.
                let boundary: HashSet<id::VertexId> =
                    e_ids.iter().flat_map(|e_id| mesh[*e_id].v).collect();
                // The verts one edge out from that boundary, excluding it.
                let neighbors: HashSet<id::VertexId> = boundary
                    .iter()
                    .flat_map(|v_id| {
                        mesh.iter_all_vert_edges(*v_id).map(|e_id| mesh[e_id].other_vert(*v_id))
                    })
                    .filter(|v_id| !boundary.contains(v_id))
                    .collect();
                // Every uncut edge spanning two of those neighbors. Each edge
                // comes up once per endpoint, so the outer `HashSet` dedupes
                // them.
                neighbors
                    .iter()
                    .flat_map(|v_id| mesh.iter_all_vert_edges(*v_id))
                    .filter(|e_id| {
                        !mesh.edge_is_cut(e_id)
                            && mesh[*e_id].v.iter().all(|v_id| neighbors.contains(v_id))
                    })
                    .map(|e_id| (*m_id, e_id))
                    .collect::<Vec<_>>()
            })
            .collect();

        if expanded.is_empty() || expanded == self.selection.edges {
            return false;
        }
        self.select_all(SelectionActionType::Deselect);
        expanded.iter().for_each(|id| {
            self.select_edge(id, SelectionActionType::Select, false, true);
        });
        true
    }

    /// Returns all the pieces which have at least one face selected in the mesh
    pub fn get_selected_pieces(&self) -> Vec<(MeshId, id::FaceId)> {
        self.selection
            .faces
            .iter()
            .filter_map(|(m_id, f_id)| self.meshes[*m_id][*f_id].p.map(|p_id| (*m_id, p_id)))
            .unique()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::face::FaceDescriptor;

    /// Side length, in verts, of the test grid.
    const N: usize = 5;

    /// A flat 5x5 grid of verts, triangulated with every quad split along its
    /// `(x, y) -> (x + 1, y + 1)` diagonal. So a vert's neighbors are the four
    /// axis-aligned ones plus the two along that diagonal.
    fn grid() -> (State, MeshId) {
        let mut mesh = crate::mesh::Mesh::new("GRID".to_string());
        let verts: Vec<_> = (0..N)
            .flat_map(|y| (0..N).map(move |x| (x, y)))
            .map(|(x, y)| mesh.add_vertex([x as f32, y as f32, 0.0]))
            .collect();
        let at = |x: usize, y: usize| verts[y * N + x];
        for y in 0..(N - 1) {
            for x in 0..(N - 1) {
                mesh.add_face(
                    &[at(x, y), at(x + 1, y), at(x + 1, y + 1)],
                    &FaceDescriptor::default(),
                );
                mesh.add_face(
                    &[at(x, y), at(x + 1, y + 1), at(x, y + 1)],
                    &FaceDescriptor::default(),
                );
            }
        }
        let mut state = State::default();
        let m_id = state.meshes.insert(mesh);
        (state, m_id)
    }

    /// The vert at grid position `(x, y)`.
    fn v(x: usize, y: usize) -> id::VertexId {
        id::VertexId::from_usize(y * N + x)
    }

    /// The selected edges, as the grid positions of their two endpoints, sorted
    /// so the set can be compared literally.
    fn selected(state: &State, m_id: MeshId) -> Vec<[(usize, usize); 2]> {
        let pos = |v_id: id::VertexId| (v_id.to_usize() % N, v_id.to_usize() / N);
        let mut edges: Vec<_> = state
            .selection
            .edges
            .iter()
            .map(|(_, e_id)| {
                let [a, b] = state.meshes[m_id][*e_id].v;
                let (a, b) = (pos(a), pos(b));
                if a < b {
                    [a, b]
                } else {
                    [b, a]
                }
            })
            .collect();
        edges.sort();
        edges
    }

    fn select_edge_between(state: &mut State, m_id: MeshId, a: (usize, usize), b: (usize, usize)) {
        let e_id = state.meshes[m_id].query_edge(v(a.0, a.1), v(b.0, b.1)).unwrap();
        state.select_edge(&(m_id, e_id), SelectionActionType::Select, false, true);
    }

    /// A single interior edge expands into the closed loop of edges one step out
    /// from its two verts, and the edge itself is left deselected.
    #[test]
    fn an_edge_expands_into_the_loop_around_it() {
        let (mut state, m_id) = grid();
        select_edge_between(&mut state, m_id, (2, 2), (3, 2));

        assert!(state.expand_selection());
        assert_eq!(
            selected(&state, m_id),
            vec![
                [(1, 1), (1, 2)],
                [(1, 1), (2, 1)],
                [(1, 2), (2, 3)],
                [(2, 1), (3, 1)],
                [(2, 3), (3, 3)],
                [(3, 1), (4, 2)],
                [(3, 3), (4, 3)],
                [(4, 2), (4, 3)],
            ]
        );
    }

    /// Already-cut edges are left out of the expansion.
    #[test]
    fn an_expansion_skips_cut_edges() {
        let (mut state, m_id) = grid();
        let cut = state.meshes[m_id].query_edge(v(1, 1), v(2, 1)).unwrap();
        state.meshes[m_id].make_cut(cut, crate::mesh::cut::CutUpdate::PiecesAndFlaps);
        select_edge_between(&mut state, m_id, (2, 2), (3, 2));

        assert!(state.expand_selection());
        let edges = selected(&state, m_id);
        assert!(!edges.contains(&[(1, 1), (2, 1)]), "{edges:?} should skip the cut edge");
        // The rest of the loop is untouched
        assert!(edges.contains(&[(1, 1), (1, 2)]));
        assert_eq!(edges.len(), 7);
    }

    /// Expanding again reaches the grid's border. The verts of the original
    /// selection are neighbors of the new boundary too, so the edge we started
    /// from comes back with it.
    #[test]
    fn expanding_twice_reaches_the_border() {
        let (mut state, m_id) = grid();
        select_edge_between(&mut state, m_id, (2, 2), (3, 2));
        assert!(state.expand_selection());
        assert!(state.expand_selection());

        let edges = selected(&state, m_id);
        assert!(edges.contains(&[(0, 0), (1, 0)]), "{edges:?} should reach the border");
        assert!(edges.contains(&[(2, 2), (3, 2)]), "{edges:?} should re-include the first edge");
        assert!(
            !edges.iter().any(|[a, b]| *a == (1, 1) && *b == (2, 1)),
            "{edges:?} should have left the first loop behind"
        );
    }

    /// Nothing to expand from, and nothing to expand into, are both no-ops so
    /// that no empty entry lands on the undo stack.
    #[test]
    fn an_expansion_with_no_result_changes_nothing() {
        let (mut state, _) = grid();
        assert!(!state.expand_selection());

        // A lone triangle has only its third vert outside the boundary, and a
        // single vert has no edge to select.
        let mut state = State::default();
        let m_id = state.meshes.insert(crate::mesh::Mesh::new_tri());
        let e_id = state.meshes[m_id].query_edge(v(0, 0), v(1, 0)).unwrap();
        state.select_edge(&(m_id, e_id), SelectionActionType::Select, false, true);
        let before = selected(&state, m_id);
        assert!(!state.expand_selection());
        assert_eq!(selected(&state, m_id), before);
    }
}
