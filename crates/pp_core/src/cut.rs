use crate::{
    id::{self},
    mesh::MeshElementType,
    State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutActionType {
    Join,
    Cut,
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CutMaskType {
    None,
    SelectionBorder,
}

impl State {
    /// Cuts multiple edges at once
    /// TODO: Batch updates together so we can more intelligently calculate new
    /// / deleted pieces, without just iterating over edges one-by-one.
    pub fn cut_edges(
        &mut self,
        edges: &[(id::MeshId, id::EdgeId)],
        action: CutActionType,
        mask: CutMaskType,
    ) {
        edges.iter().for_each(|id| {
            let (m_id, e_id) = *id;
            if action == CutActionType::Join
                || match mask {
                    CutMaskType::None => true,
                    CutMaskType::SelectionBorder => {
                        let mesh = &self.meshes[&m_id];
                        !mesh.iter_edge_loops(e_id).is_some_and(|mut walker| {
                            walker.all(|l| self.selection.faces.contains(&(m_id, mesh[l].f)))
                        })
                    }
                }
            {
                self.cut_edge(id, action)
            }
        });
    }

    /// Cuts a single edge.
    /// TODO: "Cut" any border edges during preprocessing, e.g. without two adjacent faces
    pub fn cut_edge(&mut self, id: &(id::MeshId, id::EdgeId), action: CutActionType) {
        let (m_id, e_id) = id;
        let Some(mesh) = self.meshes.get_mut(m_id) else {
            return;
        };

        let is_already_cut = mesh[*e_id].is_cut;
        let should_be_cut = match action {
            CutActionType::Join => false,
            CutActionType::Cut => true,
            CutActionType::Invert => !is_already_cut,
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
        let f_a = loops.as_mut().and_then(|l| l.next()).map(|l_id| mesh[l_id].f);
        let f_b = loops.as_mut().and_then(|l| l.next()).map(|l_id| mesh[l_id].f);
        let (Some(f_a), Some(f_b)) = (f_a, f_b) else {
            return;
        };

        // Perform the cut
        mesh[*e_id].is_cut = should_be_cut;
        mesh.elem_dirty |= MeshElementType::EDGES;

        // Now, we need to update any pieces affected by the cut / join.
        let (p_a, p_b) = (mesh[f_a].p, mesh[f_b].p);
        if should_be_cut {
            // In case of cut
            match (p_a, p_b) {
                // If faces were from the same piece, split off a new piece.
                // Make sure that the new piece does not take the root of the
                // prior piece, which would cause issues.
                (Some(p_a), Some(_)) => {
                    let new_root_f_id = if mesh[p_a].f == f_a { f_b } else { f_a };
                    mesh.add_piece(new_root_f_id).unwrap();
                }
                // If neither face was in a piece, check if we can *make* new pieces
                // starting from either piece. This most commonly returns an error,
                // which is to be expected.
                (None, None) => {
                    _ = mesh.add_piece_if_not_exists(f_a);
                    _ = mesh.add_piece_if_not_exists(f_b);
                }
                // "Cut" between different pieces isn't possible, that edge is always cut
                _ => {}
            }
        } else {
            // In case of join
            match (p_a, p_b) {
                (Some(p_a), Some(p_b)) => {
                    // If faces were from the same piece, we're in a bit of trouble.
                    // We need to traverse the piece outwards from each face and
                    // induce a cut as soon as the two pointers cross paths.
                    if p_a == p_b {
                        log::error!("Tried to join same-piece - not handled yet!")
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
}
