use std::collections::BTreeMap;

use crate::{
    id::{FaceId, LoopId, VertexId},
    mesh::cut::FlapPosition,
};

impl super::Mesh {
    /// Chooses which side of each cut on a piece's boundary carries the flap.
    ///
    /// Flaps are decided per contiguous *run* of boundary edges rather than per
    /// edge. `FlapPosition` resolves against an edge's vertex ordering, which is
    /// just whatever order `add_edge` happened to see, so left to its default a
    /// flap flips sides every other edge along a seam. A run here is a set of the
    /// piece's boundary cuts joined at shared vertices whose faces across the cut
    /// all belong to the same piece — or all belong to no piece at all.
    ///
    /// Runs facing faces which aren't in a piece yet pull the flap onto *this*
    /// piece: there is nothing on the other side to carry it. Runs facing an
    /// existing piece keep the side they already sit on, normalized by majority
    /// so that the whole run agrees.
    pub fn assign_piece_flaps(&mut self, root_f_id: FaceId) {
        // The piece-side loop of every cut on the piece's boundary. Mesh border
        // edges are skipped: with a single radial loop there is no face across
        // to flap onto. Same test `loop_has_flap` uses.
        let boundary: Vec<LoopId> = self
            .iter_connected_faces(root_f_id)
            .flat_map(|f_id| self.iter_face_loops(f_id))
            .filter(|l_id| self[*l_id].radial_next != *l_id && self.edge_is_cut(&self[*l_id].e))
            .collect();
        if boundary.is_empty() {
            return;
        }
        // The piece on the far side of each boundary cut, `None` if that face
        // isn't in a piece yet.
        let across: Vec<Option<FaceId>> =
            boundary.iter().map(|l_id| self[self[self[*l_id].radial_next].f].p).collect();

        // Union boundary cuts which meet at a vertex and face the same piece.
        // BTreeMap so runs come out the same way on every client.
        let mut at_vert: BTreeMap<VertexId, Vec<usize>> = BTreeMap::new();
        for (i, l_id) in boundary.iter().enumerate() {
            let e = self[self[*l_id].e];
            at_vert.entry(e.v[0]).or_default().push(i);
            at_vert.entry(e.v[1]).or_default().push(i);
        }
        let mut parent: Vec<usize> = (0..boundary.len()).collect();
        for meeting in at_vert.values() {
            for (n, a) in meeting.iter().copied().enumerate() {
                for b in meeting[n + 1..].iter().copied() {
                    if across[a] != across[b] {
                        continue;
                    }
                    let (root_a, root_b) = (find(&mut parent, a), find(&mut parent, b));
                    if root_a != root_b {
                        parent[root_a] = root_b;
                    }
                }
            }
        }
        let mut runs: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for i in 0..boundary.len() {
            let root = find(&mut parent, i);
            runs.entry(root).or_default().push(i);
        }

        for members in runs.into_values() {
            let faces_a_piece = across[members[0]].is_some();
            // `BothFaces` / `None` are deliberate choices, never a default, so
            // they're left exactly as the user set them.
            let members: Vec<usize> = members
                .into_iter()
                .filter(|i| {
                    self.cuts.get(&self[boundary[*i]].e).is_some_and(|cut| {
                        matches!(
                            cut.flap_position,
                            FlapPosition::FirstFace | FlapPosition::SecondFace
                        )
                    })
                })
                .collect();
            if members.is_empty() {
                continue;
            }
            let on_this_piece = if faces_a_piece {
                // Keep whichever side the run mostly sits on already; ties fall
                // to this piece. Running this from the neighbor's point of view
                // reaches the same answer, so repeated passes are stable.
                let ours = members.iter().filter(|i| self.loop_has_flap(boundary[**i])).count();
                ours * 2 >= members.len()
            } else {
                true
            };
            for i in members {
                let l_id = boundary[i];
                let mut flap_position = self.flap_position_over_loop(l_id);
                if !on_this_piece {
                    flap_position = flap_position.opposite();
                }
                self.set_cut_flap(self[l_id].e, flap_position);
            }
        }
    }

    /// Re-derives the flaps of every piece across the cut boundary of the region
    /// containing `f_id`. Used once that region stops being a piece: its
    /// neighbors now have to carry the flaps on the seams they share with it.
    pub fn assign_flaps_of_neighbors(&mut self, f_id: FaceId) {
        let mut roots: Vec<FaceId> = self
            .iter_connected_faces(f_id)
            .flat_map(|f_id| self.iter_face_loops(f_id))
            .filter(|l_id| self[*l_id].radial_next != *l_id && self.edge_is_cut(&self[*l_id].e))
            .filter_map(|l_id| self[self[self[l_id].radial_next].f].p)
            .collect();
        roots.sort();
        roots.dedup();
        roots.iter().for_each(|root| self.assign_piece_flaps(*root));
    }
}

/// Union-find root of `i`, halving the path as it climbs.
fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

#[cfg(test)]
mod tests {
    use crate::commands::{make_cuts::MakeCutsCommand, Command};
    use crate::id::{EdgeId, Id, VertexId};
    use crate::mesh::cut::CutUpdate;
    use crate::mesh::cut::FlapPosition;
    use crate::select::SelectionActionType;

    /// The cube's vertices, in the order `new_cube` adds them: the z=0 quad
    /// `v0..v3` then the z=1 quad `v4..v7` directly above it.
    fn v(i: usize) -> VertexId {
        VertexId::from_usize(i)
    }

    /// The four edges ringing the cube's bottom quad. Cutting them frees the
    /// bottom's two triangles into a piece; the other ten faces still have
    /// cycles in them, so they stay piece-less.
    const BOTTOM_RING: [(usize, usize); 4] = [(0, 1), (1, 2), (2, 3), (3, 0)];
    /// The rest of the front quad's ring, once the bottom is already cut.
    const FRONT_RING: [(usize, usize); 3] = [(0, 4), (4, 5), (5, 1)];

    fn cube() -> (crate::State, crate::MeshId) {
        let state = crate::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        (state, m_id)
    }

    fn edges(state: &crate::State, m_id: crate::MeshId, ring: &[(usize, usize)]) -> Vec<EdgeId> {
        ring.iter().map(|(a, b)| state.meshes[m_id].query_edge(v(*a), v(*b)).unwrap()).collect()
    }

    fn cut(state: &mut crate::State, m_id: crate::MeshId, ring: &[(usize, usize)]) {
        for e_id in edges(state, m_id, ring) {
            state.meshes[m_id].make_cut(e_id, CutUpdate::PiecesAndFlaps);
        }
    }

    /// Whether the flap on `e_id` sits over the face belonging to piece `root`.
    fn flap_is_on(state: &crate::State, m_id: crate::MeshId, e_id: EdgeId, root: FaceId) -> bool {
        let mesh = &state.meshes[m_id];
        mesh.iter_edge_loops(e_id)
            .unwrap()
            .find(|l_id| mesh[mesh[*l_id].f].p == Some(root))
            .is_some_and(|l_id| mesh.loop_has_flap(l_id))
    }

    fn only_piece(state: &crate::State, m_id: crate::MeshId) -> FaceId {
        let mut pieces = state.meshes[m_id].iter_pieces();
        let root = *pieces.next().expect("a piece should have been created");
        assert!(pieces.next().is_none(), "test premise: exactly one piece");
        root
    }

    use crate::id::FaceId;

    /// Rule 1: a seam facing faces which aren't in a piece yet has nothing on the
    /// far side to carry the flap, so the whole seam pulls onto the piece.
    #[test]
    fn a_seam_facing_unpieced_faces_keeps_its_flaps() {
        let (mut state, m_id) = cube();
        cut(&mut state, m_id, &BOTTOM_RING);
        let root = only_piece(&state, m_id);

        for e_id in edges(&state, m_id, &BOTTOM_RING) {
            assert!(
                flap_is_on(&state, m_id, e_id, root),
                "{e_id:?} should flap onto the piece, not onto the unpieced rest of the cube"
            );
        }
    }

    /// Rule 2: the default `FlapPosition` resolves against each edge's arbitrary
    /// vertex ordering, so left alone a seam alternates sides. Starting from a
    /// deliberately alternating layout, the run must come back uniform.
    #[test]
    fn a_run_of_boundary_cuts_lands_on_one_side() {
        let (mut state, m_id) = cube();
        let ring = edges(&state, m_id, &BOTTOM_RING);
        for (i, e_id) in ring.iter().enumerate() {
            state.meshes[m_id].make_cut(*e_id, CutUpdate::Nothing);
            let alternating =
                if i % 2 == 0 { FlapPosition::FirstFace } else { FlapPosition::SecondFace };
            state.meshes[m_id].set_cut_flap(*e_id, alternating);
        }
        // Nothing has made a piece yet, so re-cut the last edge to run the pass
        state.meshes[m_id].clear_cut(&ring[3], CutUpdate::Nothing);
        state.meshes[m_id].make_cut(ring[3], CutUpdate::PiecesAndFlaps);
        let root = only_piece(&state, m_id);

        let sides: Vec<bool> =
            ring.iter().map(|e_id| flap_is_on(&state, m_id, *e_id, root)).collect();
        assert!(sides.iter().all(|on| *on), "the run should be uniform, got {sides:?}");
    }

    /// A seam shared with a piece which already exists keeps the side it is on —
    /// including a side the user picked by hand.
    #[test]
    fn a_seam_shared_with_an_existing_piece_is_left_alone() {
        let (mut state, m_id) = cube();
        cut(&mut state, m_id, &BOTTOM_RING);
        let bottom = only_piece(&state, m_id);
        let shared = state.meshes[m_id].query_edge(v(0), v(1)).unwrap();
        assert!(flap_is_on(&state, m_id, shared, bottom), "test premise: the bottom took the flap");

        // Hand the flap to the front quad by hand, then let the front become a
        // piece: its own pass must not claw the flap back.
        let manual = state.meshes[m_id].cuts[&shared].flap_position.opposite();
        state.meshes[m_id].set_cut_flap(shared, manual);
        cut(&mut state, m_id, &FRONT_RING);
        assert!(state.meshes[m_id].iter_pieces().count() == 2, "the front should be a piece now");

        assert!(
            !flap_is_on(&state, m_id, shared, bottom),
            "the manual choice to flap onto the front should have survived"
        );
        assert_eq!(
            u8::from(state.meshes[m_id].cuts[&shared].flap_position),
            u8::from(manual),
            "and the position itself should be untouched"
        );
    }

    /// Cutting rewrites flaps as a side effect, so `MakeCutsCommand` records them
    /// and undo puts the exact previous layout back.
    #[test]
    fn undoing_a_cut_restores_the_previous_flaps() {
        let (mut state, m_id) = cube();
        cut(&mut state, m_id, &BOTTOM_RING);
        let shared = state.meshes[m_id].query_edge(v(0), v(1)).unwrap();
        let manual = state.meshes[m_id].cuts[&shared].flap_position.opposite();
        state.meshes[m_id].set_cut_flap(shared, manual);

        let before: Vec<(EdgeId, u8)> = state.meshes[m_id]
            .cuts
            .iter()
            .map(|(e_id, cut)| (*e_id, u8::from(cut.flap_position)))
            .collect();

        for e_id in edges(&state, m_id, &FRONT_RING) {
            state.select_edge(&(m_id, e_id), SelectionActionType::Select, false, true);
        }
        let cmd = MakeCutsCommand::from_select(&mut state);
        assert!(!cmd.flaps_after.is_empty(), "test premise: cutting moved some flaps");
        cmd.rollback(&mut state).unwrap();

        let after: Vec<(EdgeId, u8)> = state.meshes[m_id]
            .cuts
            .iter()
            .filter(|(e_id, _)| before.iter().any(|(id, _)| id == *e_id))
            .map(|(e_id, cut)| (*e_id, u8::from(cut.flap_position)))
            .collect();
        assert_eq!(after, before, "undo should restore every flap, manual swap included");
    }
}
