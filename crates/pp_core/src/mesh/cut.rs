use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Transform, Vector3, Zero};
use serde::{Deserialize, Serialize};

use crate::{id, mesh::MeshElementType};

/// How far a piece slides out from the seam when a cut splits it off its
/// parent, in world units (1 unit = 1cm). Far enough to read as two pieces,
/// close enough that the user can still see both halves of what they cut.
const CUT_PIECE_SEPARATION: f32 = 0.5;

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
    pub(crate) fn get_adjacent_two_faces(
        &self,
        e_id: id::EdgeId,
    ) -> Option<(id::FaceId, id::FaceId)> {
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

                        // Pieces are never deleted within a session, so an entry
                        // which is already here belongs to a redo: it remembers
                        // wherever the user dragged the piece after the original
                        // cut, and re-placing it would throw that away.
                        let is_new = !self.pieces.contains_key(&face_with_new_piece);
                        self.expand_piece(face_with_new_piece).unwrap();
                        if is_new {
                            self.place_split_piece(p_1, face_with_new_piece, e_id);
                        }
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

    /// Places the piece which a cut just split off `parent_root` where its faces
    /// already were, then slides it [`CUT_PIECE_SEPARATION`] out from the seam.
    ///
    /// A piece is unfolded from its root face: `affine_final` comes off that
    /// face's normal, and every other face is placed by a chain of hinges
    /// starting there. The half which loses the old root therefore gets a brand
    /// new origin, and with the identity transform `Piece::default` hands it, it
    /// would appear at an arbitrary spot nowhere near the edge the user cut.
    ///
    /// `e_id` is the edge which was cut, and is expected to already be marked as
    /// such: the walk over the parent stops at that seam, which is exactly what
    /// keeps it on the parent's side.
    fn place_split_piece(
        &mut self,
        parent_root: id::FaceId,
        new_root: id::FaceId,
        e_id: id::EdgeId,
    ) {
        let Some(parent) = self.pieces.get(&parent_root).copied() else { return };

        // The seam's loop on the half which kept the root. The parent's unfold
        // can't reach `new_root` anymore, so we walk to the face on this side and
        // hinge across, which is the step the walk itself would have taken.
        let Some(l_stay) =
            self.iter_edge_loops(e_id).and_then(|mut ls| ls.find(|l_id| self[*l_id].f != new_root))
        else {
            return;
        };
        let f_stay = self[l_stay].f;
        let Some(affine_stay) = self
            .iter_piece_faces_unfolded(parent_root)
            .find(|face| face.f == f_stay)
            .map(|face| face.affine)
        else {
            return;
        };
        // Where `new_root` sat within the parent's unfold, before the cut.
        let affine_old = affine_stay * self.unfold_hinge_affine(l_stay, parent.t);

        // Both halves have to unfold by the same amount for the seam to line up.
        if let Some(piece) = self.pieces.get_mut(&new_root) {
            piece.t = parent.t;
        }
        // The new piece's own unfold origin. Its root face is the one the walk
        // starts at, so its affine is `affine_final` alone.
        let affine_new = self.iter_piece_faces_unfolded(new_root).affine_final;
        let Some(affine_new_inv) = affine_new.invert() else { return };
        // Undo the new origin, then re-apply the old one: every vertex of the new
        // piece lands exactly where it was drawn a moment ago.
        let align = parent.transform * affine_old * affine_new_inv;

        // Slide out along the seam's perpendicular, toward the new piece's own
        // body, so it separates from the parent instead of sliding over it.
        let place_m = align * affine_new;
        let place = |v: Vector3<f32>| place_m.transform_point(Point3::from_vec(v)).to_vec();
        let e = self[e_id];
        let (p0, p1) = (place(self.vert_pos(e.v[0])), place(self.vert_pos(e.v[1])));
        let seam = (p1 - p0).normalize();
        let mid = p0 + (p1 - p0) * 0.5;
        let mut centroid = Vector3::zero();
        let mut count = 0.0;
        for l_id in self.iter_face_loops(new_root) {
            centroid += place(self.vert_pos(self[l_id].v));
            count += 1.0;
        }
        let outward = centroid / count - mid;
        // Only the component across the seam, so the piece doesn't drift along it.
        let outward = outward - seam * outward.dot(seam);
        let offset = if outward.magnitude() > 1e-5 {
            outward.normalize() * CUT_PIECE_SEPARATION
        } else {
            Vector3::zero()
        };

        // The piece is still at the identity, so this sets the transform outright
        // while raising the dirty flags the renderer needs.
        self.transform_piece(&new_root, Matrix4::from_translation(offset) * align);
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

#[cfg(test)]
mod tests {
    use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Transform, Vector3};

    use crate::commands::{make_cuts::MakeCutsCommand, Command};
    use crate::id::{EdgeId, FaceId, Id, VertexId};
    use crate::mesh::cut::CUT_PIECE_SEPARATION;
    use crate::select::SelectionActionType;

    /// The cube's vertices, in the order `new_cube` adds them: the z=0 quad
    /// `v0..v3` then the z=1 quad `v4..v7` directly above it.
    fn v(i: usize) -> VertexId {
        VertexId::from_usize(i)
    }

    /// Frees the bottom quad and the front quad as one four-triangle strip,
    /// hinged on the edge `v0-v1` they share.
    const STRIP: [(usize, usize); 6] = [(1, 2), (2, 3), (3, 0), (0, 4), (4, 5), (5, 1)];
    /// The hinge itself. Cutting it splits the strip into two pieces, and the
    /// piece which keeps the strip's root is the one on `v0-v1`'s second face,
    /// so undoing the cut merges back into the piece the cut created.
    const HINGE: [(usize, usize); 1] = [(0, 1)];

    fn cube() -> (crate::State, crate::MeshId) {
        let state = crate::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        (state, m_id)
    }

    fn edges(state: &crate::State, m_id: crate::MeshId, ring: &[(usize, usize)]) -> Vec<EdgeId> {
        ring.iter().map(|(a, b)| state.meshes[m_id].query_edge(v(*a), v(*b)).unwrap()).collect()
    }

    /// The live piece roots, sorted so they can be compared as a set.
    fn roots(state: &crate::State, m_id: crate::MeshId) -> Vec<FaceId> {
        let mut roots: Vec<FaceId> = state.meshes[m_id].iter_pieces().copied().collect();
        roots.sort();
        roots
    }

    fn transform_of(state: &crate::State, m_id: crate::MeshId, root: FaceId) -> Matrix4<f32> {
        state.meshes[m_id].pieces[&root].transform
    }

    fn cut(
        state: &mut crate::State,
        m_id: crate::MeshId,
        ring: &[(usize, usize)],
    ) -> MakeCutsCommand {
        for e_id in edges(state, m_id, ring) {
            state.select_edge(&(m_id, e_id), SelectionActionType::Select, false, true);
        }
        let cmd = MakeCutsCommand::from_select(state);
        state.select_all(SelectionActionType::Deselect);
        cmd
    }

    /// Cuts the strip into a piece and then splits it in two, returning the
    /// piece which was already there, the one the split created, and the
    /// command which split them.
    fn split() -> (crate::State, crate::MeshId, FaceId, FaceId, MakeCutsCommand) {
        let (mut state, m_id) = cube();
        cut(&mut state, m_id, &STRIP);
        let existing = *state.meshes[m_id].iter_pieces().next().expect("the strip is a piece");

        let cmd = cut(&mut state, m_id, &HINGE);
        let created = *roots(&state, m_id)
            .iter()
            .find(|f_id| **f_id != existing)
            .expect("test premise: the split made a second piece");
        (state, m_id, existing, created, cmd)
    }

    /// A piece is identified by its root face, and both its transform and its
    /// unfold origin hang off that face. Undoing this split merges into the
    /// piece the cut created, so without the recorded roots the redo would root
    /// the other half at whichever face the cut reaches first, handing it a
    /// fresh identity transform and stranding the one the user had moved.
    #[test]
    fn redoing_a_cut_restores_the_moved_pieces() {
        let (mut state, m_id, existing, created, cmd) = split();
        // The cut already placed the piece it created, so what has to survive the
        // round trip is that placement with the user's drag stacked on top.
        let (placed_existing, placed_created) =
            (transform_of(&state, m_id, existing), transform_of(&state, m_id, created));
        let (moved_existing, moved_created) = (
            Matrix4::from_translation(cgmath::vec3(1.0, 2.0, 0.0)),
            Matrix4::from_translation(cgmath::vec3(-3.0, 4.0, 0.0)),
        );
        state.meshes[m_id].transform_piece(&existing, moved_existing);
        state.meshes[m_id].transform_piece(&created, moved_created);
        let (want_existing, want_created) =
            (moved_existing * placed_existing, moved_created * placed_created);

        cmd.rollback(&mut state).unwrap();
        assert_eq!(
            roots(&state, m_id),
            vec![existing],
            "undo should merge back into the piece which was there before the cut"
        );
        assert_eq!(
            transform_of(&state, m_id, existing),
            want_existing,
            "and leave that piece where it was"
        );

        cmd.execute(&mut state).unwrap();
        let mut expected = vec![existing, created];
        expected.sort();
        assert_eq!(roots(&state, m_id), expected, "redo should root both pieces where they were");
        assert_eq!(transform_of(&state, m_id, existing), want_existing);
        assert_eq!(
            transform_of(&state, m_id, created),
            want_created,
            "the piece the cut created should come back where the user moved it, \
             not be re-placed off the seam a second time"
        );
    }

    /// The same round trip on untouched pieces: the roots have to survive it
    /// even when nothing was moved, since a `TransformPiecesCommand` later in
    /// the stack addresses its pieces by root face.
    #[test]
    fn undo_redo_leaves_the_roots_alone() {
        let (mut state, m_id, existing, created, cmd) = split();
        let placed = transform_of(&state, m_id, created);
        assert_ne!(placed, Matrix4::identity(), "the cut places the piece it splits off");

        cmd.rollback(&mut state).unwrap();
        cmd.execute(&mut state).unwrap();

        let mut expected = vec![existing, created];
        expected.sort();
        assert_eq!(roots(&state, m_id), expected);
        assert_eq!(
            transform_of(&state, m_id, created),
            placed,
            "and the round trip should leave that placement alone"
        );
    }

    /// Where the piece rooted at `root` draws vertex `v_id` of face `f_id`:
    /// the piece transform on top of that face's unfolding.
    fn placed(
        state: &crate::State,
        m_id: crate::MeshId,
        root: FaceId,
        f_id: FaceId,
        v_id: VertexId,
    ) -> Vector3<f32> {
        let mesh = &state.meshes[m_id];
        let face = mesh
            .iter_piece_faces_unfolded(root)
            .find(|face| face.f == f_id)
            .expect("the face belongs to the piece");
        (mesh.pieces[&root].transform * face.affine)
            .transform_point(Point3::from_vec(mesh.vert_pos(v_id)))
            .to_vec()
    }

    /// A piece is unfolded from its root face, so the half a cut splits off gets
    /// a brand new origin and used to land wherever that origin happened to fall.
    /// It should instead stay on the seam it was cut from, backed off by exactly
    /// `CUT_PIECE_SEPARATION` so both halves stay visible.
    #[test]
    fn a_cut_shifts_the_new_piece_just_off_the_seam() {
        let (state, m_id, existing, created, _) = split();
        let e_id = edges(&state, m_id, &HINGE)[0];
        let (f_a, f_b) = state.meshes[m_id].get_adjacent_two_faces(e_id).unwrap();
        let (f_new, f_old) =
            if state.meshes[m_id][f_a].p == Some(created) { (f_a, f_b) } else { (f_b, f_a) };

        let e = state.meshes[m_id][e_id];
        let seam = placed(&state, m_id, existing, f_old, e.v[1])
            - placed(&state, m_id, existing, f_old, e.v[0]);
        let offsets: Vec<Vector3<f32>> =
            e.v.iter()
                .map(|v_id| {
                    placed(&state, m_id, created, f_new, *v_id)
                        - placed(&state, m_id, existing, f_old, *v_id)
                })
                .collect();

        for offset in &offsets {
            assert!(
                (offset.magnitude() - CUT_PIECE_SEPARATION).abs() < 1e-4,
                "each end of the seam should sit {CUT_PIECE_SEPARATION}cm from where the \
                 parent still draws it, but this one moved {}cm",
                offset.magnitude()
            );
            assert!(
                offset.dot(seam.normalize()).abs() < 1e-4,
                "the piece should back straight off the seam, not slide along it"
            );
        }
        assert!(
            (offsets[0] - offsets[1]).magnitude() < 1e-4,
            "the seam should come away whole: both ends move by the same vector"
        );
    }
}
