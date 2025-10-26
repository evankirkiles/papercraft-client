use serde::{Deserialize, Serialize};

use crate::{id, mesh::MeshElementType};

// State of an edge's cut
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cut {
    /// Which loop / face the flap extends to
    pub flap_position: FlapPosition,
    /// If `true`, then this `Cut` is inactive, but kept around for undo / redo
    pub is_dead: bool,
}

#[repr(u8)]
#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize)]
pub enum FlapPosition {
    #[default]
    FirstFace,
    SecondFace,
    BothFaces,
    None,
}

impl From<u8> for FlapPosition {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::FirstFace,
            1 => Self::SecondFace,
            2 => Self::BothFaces,
            3 => Self::None,
            _ => Self::FirstFace,
        }
    }
}

impl From<FlapPosition> for u8 {
    fn from(val: FlapPosition) -> Self {
        match val {
            FlapPosition::FirstFace => 0,
            FlapPosition::SecondFace => 1,
            FlapPosition::BothFaces => 2,
            FlapPosition::None => 3,
        }
    }
}

impl super::Mesh {
    // Extract the adjacent faces to the edge. Technically it's possible for
    // the mesh to have more than 2 faces per edge, but we can preprocess
    // that invariant out, so I don't want to try to support that use case.
    // Similarly, if the edge had <2 faces, it's either a boundary or
    // dangling, in which case "cutting" doesn't make much sense either.
    // Faces are in radial order, so A then B in the radial link
    fn get_adjacent_two_faces(&self, e_id: id::EdgeId) -> Option<(id::FaceId, id::FaceId)> {
        let mut adj_faces = self.iter_edge_loops(e_id).map(|it| it.map(|l_id| self[l_id].f));
        let f_1 = adj_faces.as_mut().and_then(|faces| faces.next());
        let f_2 = adj_faces.as_mut().and_then(|faces| faces.next());
        if let (Some(f_1), Some(f_2)) = (f_1, f_2) {
            Some((f_1, f_2))
        } else {
            None
        }
    }

    /// Adds / restores a cut on an edge.
    pub fn make_cut(&mut self, e_id: id::EdgeId, update_pieces: bool) {
        self.cuts.entry(e_id).or_default().is_dead = false;
        self.elem_dirty |= MeshElementType::EDGES;
        if !update_pieces {
            return;
        }
        // What we're interested in are the pieces of each adjacent face
        if let Some((f_1, f_2)) = self.get_adjacent_two_faces(e_id) {
            let (p_1, p_2) = (self[f_1].p, self[f_2].p);
            match (p_1, p_2) {
                // If faces were from the same piece, create a new piece starting
                // from the face which no longer has a path back to the piece root.
                // Note that this branch also handles the cut between two
                // different pieces, but that must ALREADY be a cut, so it
                // should never happen.
                (Some(p_1), Some(p_2)) => {
                    if p_1 == p_2 {
                        let face_with_new_piece = self
                            .iter_connected_faces(f_1)
                            .find(|f_id| f_id == &p_1)
                            .map(|_| f_2) // If p_1 is found, then piece starts at f_2
                            .unwrap_or(f_1); // Otherwise, the piece starts at f_1
                        self.expand_piece(face_with_new_piece).unwrap();
                    }
                }
                // If neither face was in a piece, check if we can *make* new pieces
                // starting from either piece. We only create a new piece for f_2
                // if it doesn't get brought into a piece from f_1.
                (None, None) => {
                    let _ = self.expand_piece(f_1);
                    if self[f_2].p.is_none() {
                        let _ = self.expand_piece(f_2);
                    }
                }
                // "Cut" between different pieces isn't possible, the edge must already be cut
                _ => {}
            }
        };
        let pieces: Vec<_> = self.iter_pieces().collect();
        log::info!("Piece Count: {:?}", pieces.len());
        for f_id in pieces {
            log::info!("Piece {:?}", f_id)
        }
    }

    /// Removes the cut on an edge. Note that the internal cut state persists
    /// in the state, but is marked with a tombstone so it is treated as "uncut".
    pub fn clear_cut(&mut self, e_id: &id::EdgeId, update_pieces: bool) {
        let Some(cut) = self.cuts.get_mut(e_id) else {
            return;
        };
        cut.is_dead = true;
        self.elem_dirty |= MeshElementType::EDGES;
        if !update_pieces {
            return;
        }
        // What we're interested in are the pieces of each adjacent face
        if let Some((f_1, f_2)) = self.get_adjacent_two_faces(*e_id) {
            let (p_1, p_2) = (self[f_1].p, self[f_2].p);
            match (p_1, p_2) {
                (Some(p_1), Some(p_2)) => {
                    // If faces were from the same piece, clear the piece, as the
                    // piece now must have a cycle. Our iterator needs to be able
                    // to not fall infinitely into that cycle (check this).
                    if p_1 == p_2 {
                        let _ = self.clear_piece(p_1);
                    } else {
                        // If faces were from different pieces, we can just clear
                        // one of the pieces and rope all of its faces into the
                        // other pre-existing piece. This technically also iterates
                        // over the p_b pieces too (because we remove the cut
                        // earlier), but their piece ids will remain the same.
                        let _ = self.expand_piece(p_1);
                    }
                }
                // If either face was not in a piece, then all faces involved
                // are now free-floating. We need to delete the old piece.
                (Some(p_id), None) | (None, Some(p_id)) => self.clear_piece(p_id),
                // Nothing needed if neither face was in a piece
                (None, None) => {}
            }
        };
        let pieces: Vec<_> = self.iter_pieces().collect();
        log::info!("Piece Count: {:?}", pieces.len());
        for f_id in pieces {
            log::info!("Piece {:?}", f_id)
        }
    }

    /// Sets the flap position of a cut
    pub fn set_cut_flap(&mut self, id: id::EdgeId, flap_position: FlapPosition) {
        self.cuts.entry(id).and_modify(|cut| cut.flap_position = flap_position);
        self.elem_dirty |= MeshElementType::FLAPS;
    }

    /// Tells whether an edge is cut or not
    pub fn edge_is_cut(&self, id: &id::EdgeId) -> bool {
        self.cuts.get(id).is_some_and(|cut| !cut.is_dead)
    }
}
