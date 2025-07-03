use std::collections::HashMap;

use crate::{
    id::{self},
    mesh::{self, edge::EdgeCut, MeshElementType},
    MeshId, State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutActionType {
    Join,
    Cut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutMaskType {
    None,
    SelectionBorder,
}

/// Keeps track of information we need to preserve per-edge across cuts.
/// Tuples represent data for face A and B
#[derive(Clone, Debug)]
pub struct CutEdgeState {
    pub cut: Option<EdgeCut>,
    pub p_a: Option<id::PieceId>,
    pub p_b: Option<id::PieceId>,
}

impl State {
    /// Cuts multiple edges at once
    /// TODO: Batch updates together so we can more intelligently calculate new
    /// / deleted pieces, without just iterating over edges one-by-one.
    pub fn cut_edges(
        &mut self,
        edges: &[(MeshId, id::EdgeId)],
        action: CutActionType,
        history: Option<&HashMap<(MeshId, id::EdgeId), CutEdgeState>>,
    ) {
        edges
            .iter()
            .for_each(|id| self.cut_edge(id, action, history.as_ref().and_then(|h| h.get(id))));
    }

    /// Cuts a single edge.
    /// TODO: "Cut" any border edges during preprocessing, e.g. without two adjacent faces
    pub fn cut_edge(
        &mut self,
        id: &(MeshId, id::EdgeId),
        action: CutActionType,
        history: Option<&CutEdgeState>,
    ) {
        let (m_id, e_id) = id;
        let Some(mesh) = self.meshes.get_mut(*m_id) else {
            return;
        };

        let is_already_cut = mesh[*e_id].cut.is_some();
        let should_be_cut = match action {
            CutActionType::Join => false,
            CutActionType::Cut => true,
        };

        // If the edge is already in the desired state, do nothing
        if is_already_cut == should_be_cut {
            return;
        }

        // Extract the adjacent faces to the edge. Technically it's possible for
        // the mesh to have more than 2 faces per edge, but we can preprocess
        // that invariant out, so I don't want to try to support that use case.
        // Similarly, if the edge had <2 faces, it's either a boundary or
        // dangling, in which case "cutting" doesn't make much sense either.
        let mut loops = mesh.iter_edge_loops(*e_id);
        let Some(loops) = loops.as_mut() else {
            return;
        };
        let (Some(l_a), Some(l_b)) = (loops.next(), loops.next()) else {
            return;
        };
        // Faces are in radial order, so A then B in the radial link
        let f_a = mesh[l_a].f;
        let f_b = mesh[l_b].f;

        // Perform the cut, placing the flap on the non-selected face or l_a by default
        mesh[*e_id].cut = should_be_cut.then_some(
            history.and_then(|h| h.cut).unwrap_or(mesh::edge::EdgeCut { l_flap: Some(l_a) }),
        );
        mesh.elem_dirty |= MeshElementType::EDGES;

        // Now, we need to update any pieces affected by the cut / join.
        let (p_a, p_b) = (mesh[f_a].p, mesh[f_b].p);
        if should_be_cut {
            // In case of cut
            match (p_a, p_b) {
                // If faces were from the same piece, split off a new piece at B.
                // Make sure that the new piece does not take the root of the
                // prior piece, which would cause issues.
                (Some(p_a), Some(_)) => {
                    if mesh[p_a].f != f_a {
                        let _ = mesh.create_piece(f_a, history.and_then(|h| h.p_a)).unwrap();
                    } else {
                        let _ = mesh.create_piece(f_b, history.and_then(|h| h.p_b)).unwrap();
                    }
                }
                // If neither face was in a piece, check if we can *make* new pieces
                // starting from either piece.
                (None, None) => {
                    let _ = mesh.create_piece(f_a, history.and_then(|h| h.p_a));
                    let _ = mesh.create_piece(f_b, history.and_then(|h| h.p_b));
                }
                // "Cut" between different pieces isn't possible, that edge is always cut
                _ => {}
            }
        } else {
            // In case of join
            match (p_a, p_b) {
                (Some(p_a), Some(p_b)) => {
                    // If faces were from the same piece, just clear the piece,
                    // as this now induces a cycle in the piece's surface.
                    if p_a == p_b {
                        mesh.remove_piece(p_a, None);
                    } else {
                        // If faces were from different pieces, we're chilling. We
                        // can just clear one of the pieces and rope all of its faces
                        // into the other pre-existing piece. This technically also
                        // iterates over the p_b pieces too (because we remove the
                        // cut earlier), but their piece id remains the same.
                        mesh.remove_piece(p_a, Some(p_b));
                    }
                }
                // If either face was not in a piece, then all faces involved
                // are now free-floating. We need to delete the old piece.
                (Some(p_id), None) | (None, Some(p_id)) => mesh.remove_piece(p_id, None),
                // Nothing needed if neither face was in a piece
                (None, None) => {}
            }
        }
    }

    /// Swaps the edge's flap to the other face
    pub fn swap_edge_flap(&mut self, id: &(MeshId, id::EdgeId)) {
        let mesh = self.meshes.get_mut(id.0).unwrap();
        let new_l = mesh[id.1].cut.and_then(|c| c.l_flap).map(|l_flap| mesh[l_flap].radial_next);
        if let Some(cut) = mesh[id.1].cut.as_mut() {
            cut.l_flap = new_l;
        }
        mesh.elem_dirty |= MeshElementType::FLAPS;
    }

    /// Sets / clears the edge's flap
    pub fn set_edge_flap(&mut self, id: &(MeshId, id::EdgeId), l_flap: Option<id::LoopId>) {
        let mesh = self.meshes.get_mut(id.0).unwrap();
        if let Some(cut) = mesh[id.1].cut.as_mut() {
            cut.l_flap = l_flap;
        }
        mesh.elem_dirty |= MeshElementType::FLAPS;
    }
}
