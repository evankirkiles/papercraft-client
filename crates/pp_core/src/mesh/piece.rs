use std::collections::{HashSet, VecDeque};

use crate::id::{self, Id};

/// A face, formed by three vertices and three edges.
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    /// Any face in the piece, used as the "root"
    pub f: id::FaceId,

    /// In progressive unwrapping editors, this indicates the "unwrappedness"
    /// of the piece. t=0 means fully 3d, whereas t=1 means fully 2d
    pub t: f64,

    /// Indicates if this piece's internal faces have changed
    pub elem_dirty: bool,
    /// Indicates if this piece's uniform data has changed, e.g. transform / hover state
    pub is_dirty: bool,
}

impl Piece {
    fn new(f: id::FaceId) -> Self {
        Self { f, t: 0.0, elem_dirty: false, is_dirty: false }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PieceCreationError {
    PieceExists,
    CycleDetected,
}

impl super::Mesh {
    /// Tries to create a new piece from all the faces connected to a given face.
    /// Returns an error if
    pub(crate) fn create_piece(
        &mut self,
        f_id: id::FaceId,
    ) -> Result<id::PieceId, PieceCreationError> {
        self.assert_face_can_make_piece(f_id)?;
        let piece = Piece::new(f_id);
        let p_id = id::PieceId::from_usize(self.pieces.push(piece));
        let f_ids: Vec<_> = self.iter_connected_faces(f_id).collect();
        f_ids.iter().for_each(|f_id| self[*f_id].p = Some(p_id));
        log::info!("Made piece {p_id:?} at {f_id:?}");
        Ok(p_id)
    }

    pub(crate) fn create_piece_if_not_exists(
        &mut self,
        f_id: id::FaceId,
    ) -> Result<id::PieceId, PieceCreationError> {
        if self[f_id].p.is_some() {
            return Err(PieceCreationError::PieceExists);
        }
        self.create_piece(f_id)
    }

    /// "Clears" a piece, returning all of its contained faces back to a no-piece state
    pub(crate) fn remove_piece(&mut self, p_id: id::PieceId, new_p_id: Option<id::PieceId>) {
        let f_id = self[p_id].f;
        let f_ids: Vec<_> = self.iter_connected_faces(f_id).collect();
        f_ids.iter().for_each(|f_id| self[*f_id].p = new_p_id);
        log::info!("Removed piece {p_id:?} for {new_p_id:?}");
        self.pieces.remove(p_id.to_usize());
    }

    /// Ensures that there are no cycles in faces connected to this mesh
    fn assert_face_can_make_piece(&self, f_start: id::FaceId) -> Result<(), PieceCreationError> {
        let mut frontier = VecDeque::from([(f_start, None)]);
        let mut visited = HashSet::from([f_start]);

        while let Some((f_id, parent)) = frontier.pop_front() {
            for neighbor in self
                .iter_face_loops(f_id)
                .filter_map(|l| {
                    let e_id = self[l].e;
                    let is_cut = self[e_id].is_cut;
                    if !is_cut {
                        self.iter_edge_loops(e_id)
                    } else {
                        None
                    }
                })
                .flatten()
                .filter_map(|l_id| {
                    let neighbor_id = self[l_id].f;
                    (neighbor_id != f_id).then_some(neighbor_id)
                })
            {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    frontier.push_back((neighbor, Some(f_id))); // Mark current face as the parent
                } else if Some(neighbor) != parent {
                    // If the neighbor has already been visited and is not the direct parent of f_id, a cycle exists.
                    return Err(PieceCreationError::CycleDetected);
                }
            }
        }

        Ok(())
    }
}
