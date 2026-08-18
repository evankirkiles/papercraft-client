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

impl FlapPosition {
    /// The same flap, moved onto the other face of the cut. Positions which
    /// aren't one-sided have no other side to move to, so they stay put.
    pub fn opposite(self) -> Self {
        match self {
            Self::FirstFace => Self::SecondFace,
            Self::SecondFace => Self::FirstFace,
            Self::BothFaces => Self::BothFaces,
            Self::None => Self::None,
        }
    }
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

/// How far a cut should propagate into the rest of the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutUpdate {
    /// Flip the cut flag and nothing else, for loading a saved document whose
    /// pieces and flaps are already recorded.
    Nothing,
    /// Recompute the pieces around the edge, and derive flaps for whatever
    /// changed. This is the interactive path.
    PiecesAndFlaps,
    /// Recompute the pieces, but leave flaps alone: the caller is replaying a
    /// recorded flap layout on top and would only have to undo the guesswork.
    PiecesOnly,
}

impl CutUpdate {
    fn touches_pieces(self) -> bool {
        self != Self::Nothing
    }

    fn touches_flaps(self) -> bool {
        self == Self::PiecesAndFlaps
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
    pub fn make_cut(&mut self, e_id: id::EdgeId, update: CutUpdate) {
        self.cuts.entry(e_id).or_default().is_dead = false;
        self.elem_dirty |= MeshElementType::EDGES;
        if !update.touches_pieces() {
            return;
        }
        // What we're interested in are the pieces of each adjacent face
        log::info!("Making cut at: {:?}", e_id);
        if let Some((f_1, f_2)) = self.get_adjacent_two_faces(e_id) {
            let (p_1, p_2) = (self[f_1].p, self[f_2].p);
            log::info!("Cut between pieces: {:?}, {:?}", p_1, p_2);
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
                        if update.touches_flaps() {
                            self.assign_piece_flaps(face_with_new_piece);
                            self.assign_piece_flaps(p_1);
                        }
                    }
                }
                // If neither face was in a piece, check if we can *make* new pieces
                // starting from either piece. We only create a new piece for f_2
                // if it doesn't get brought into a piece from f_1.
                (None, None) => {
                    let _ = self.expand_piece(f_1);
                    // Flaps are assigned right after each `expand_piece` rather
                    // than once at the end: f_2 is still piece-less at this
                    // point, so the seam between them lands on f_1's piece, and
                    // f_2's own pass then sees that choice and leaves it alone.
                    if update.touches_flaps() && self[f_1].p == Some(f_1) {
                        self.assign_piece_flaps(f_1);
                    }
                    if self[f_2].p.is_none() {
                        let _ = self.expand_piece(f_2);
                        if update.touches_flaps() && self[f_2].p == Some(f_2) {
                            self.assign_piece_flaps(f_2);
                        }
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
    pub fn clear_cut(&mut self, e_id: &id::EdgeId, update: CutUpdate) {
        let Some(cut) = self.cuts.get_mut(e_id) else {
            return;
        };
        cut.is_dead = true;
        self.elem_dirty |= MeshElementType::EDGES;
        if !update.touches_pieces() {
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
                        self.clear_piece(p_1);
                        // The region is no longer a piece, so its neighbors have
                        // to take over the flaps on the seams they share with it.
                        if update.touches_flaps() {
                            self.assign_flaps_of_neighbors(f_1);
                        }
                    } else {
                        // If faces were from different pieces, we can just clear
                        // one of the pieces and rope all of its faces into the
                        // other pre-existing piece. This technically also iterates
                        // over the p_b pieces too (because we remove the cut
                        // earlier), but their piece ids will remain the same.
                        let _ = self.expand_piece(p_1);
                        if update.touches_flaps() && self[p_1].p == Some(p_1) {
                            self.assign_piece_flaps(p_1);
                        }
                    }
                }
                // If either face was not in a piece, then all faces involved
                // are now free-floating. We need to delete the old piece.
                (Some(p_id), None) | (None, Some(p_id)) => {
                    self.clear_piece(p_id);
                    if update.touches_flaps() {
                        self.assign_flaps_of_neighbors(f_1);
                    }
                }
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

    /// Tells whether the flap for this loop's edge extends over this loop's face.
    /// The two radial loops of an edge start at opposite endpoints, so `l.v` is
    /// what picks out which side of the cut we're on.
    pub fn loop_has_flap(&self, l_id: id::LoopId) -> bool {
        let l = self[l_id];
        // Boundary edge: only one radial loop, so there's nothing to flap onto
        if l_id == l.radial_next {
            return false;
        }
        let e = self[l.e];
        self.cuts.get(&l.e).is_some_and(|cut| {
            !cut.is_dead
                && match cut.flap_position {
                    FlapPosition::FirstFace => l.v != e.v[0],
                    FlapPosition::SecondFace => l.v != e.v[1],
                    FlapPosition::BothFaces => true,
                    FlapPosition::None => false,
                }
        })
    }

    /// The `FlapPosition` which puts the flap on this loop's face. This is the
    /// only place which knows that `FirstFace` means `l.v != e.v[0]`, i.e. that
    /// it names the loop starting at `e.v[1]`.
    pub fn flap_position_over_loop(&self, l_id: id::LoopId) -> FlapPosition {
        let l = self[l_id];
        if l.v == self[l.e].v[1] {
            FlapPosition::FirstFace
        } else {
            FlapPosition::SecondFace
        }
    }
}
