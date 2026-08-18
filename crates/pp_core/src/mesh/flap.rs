use std::collections::BTreeMap;

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, Transform, Vector3};

use crate::{
    id::{FaceId, LoopId, VertexId},
    mesh::cut::FlapPosition,
};

/// How far a flap may reach off the edge it hangs from, in centimeters.
///
/// A cap rather than a proportion: tabs are glued by hand, so past a few
/// millimeters extra reach stops helping and just eats page area.
pub const MAX_FLAP_HEIGHT: f32 = 0.3;

/// The widest half-angle a flap's corner may take, in radians (45°).
///
/// The flap is inscribed in the isosceles triangle standing on its base edge,
/// which for a very obtuse neighbouring face would run away to a long spike.
pub const MAX_FLAP_APEX_ANGLE: f32 = std::f32::consts::FRAC_PI_4;

/// The four corners of the trapezoid flap hanging off the edge `v0 -> v1`,
/// as `[v0, v1, top1, top0]` — bottom-left, bottom-right, top-right, top-left.
///
/// `v2` is the anchor: the unfolded position of the vertex across the cut, whose
/// face tells the flap which way to lean and how much room it has. All three
/// points are in the same unfolded piece-local space, so the caller still has to
/// apply the piece transform to place the result on a page.
///
/// The flap is inscribed in the isosceles triangle standing on `v0 -> v1`, with
/// its apex angle capped at [`MAX_FLAP_APEX_ANGLE`] and its height at
/// [`MAX_FLAP_HEIGHT`], then cut off parallel to the base. Capping the angle
/// before the height is what keeps the tab inside the facing face: a flap that
/// leaned out further than the face it folds onto would stick out past the
/// silhouette once the model is assembled.
///
/// This is the single definition of flap geometry. The GPU reads the corners it
/// returns rather than deriving its own, and the vector print path strokes their
/// outline, so the printed tab and the tab on screen cannot drift apart.
///
/// Degenerate input — a zero-length base, or an anchor collinear with it — has
/// no well-defined lean, so the flap collapses onto its base and renders as
/// nothing.
pub fn flap_corners(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>) -> [Point3<f32>; 4] {
    let collapsed =
        || [Point3::from_vec(v0), Point3::from_vec(v1), Point3::from_vec(v1), Point3::from_vec(v0)];

    // The direction the isosceles triangle extends: perpendicular to the base,
    // in the plane the base and the anchor span.
    let base_vec = v1 - v0;
    let base_len = base_vec.magnitude();
    let tri_normal = base_vec.cross(v2 - v0);
    if base_len <= f32::EPSILON || tri_normal.magnitude() <= f32::EPSILON {
        return collapsed();
    }
    let base_dir = base_vec / base_len;
    let base_mid = 0.5 * (v0 + v1);
    let perp_dir = tri_normal.normalize().cross(base_dir);

    // The apex of the triangle the flap is inscribed in, leaning no further than
    // the shallower of the facing face's two base corners.
    let angle_at = |a: Vector3<f32>, b: Vector3<f32>| {
        ((b - a).normalize().dot((v2 - a).normalize())).clamp(-1.0, 1.0).acos()
    };
    let min_angle = MAX_FLAP_APEX_ANGLE.min(angle_at(v0, v1)).min(angle_at(v1, v0));
    let height = 0.5 * base_len * min_angle.tan();
    if !height.is_finite() || height <= f32::EPSILON {
        return collapsed();
    }
    let apex = base_mid + perp_dir * height;

    // Truncate the triangle to the height cap, keeping the sides' slope so every
    // tab in the document leans the same way its own geometry asks for.
    let depth_scale = height.min(MAX_FLAP_HEIGHT) / height;
    let top0 = v0 + (apex - v0) * depth_scale;
    let top1 = v1 + (apex - v1) * depth_scale;
    [Point3::from_vec(v0), Point3::from_vec(v1), Point3::from_vec(top1), Point3::from_vec(top0)]
}

impl super::Mesh {
    /// The four corners of the flap hanging over loop `l_id`, or `None` if that
    /// side of the edge carries no flap.
    ///
    /// `affine` is the unfolding transform of `l_id`'s own face, as handed out by
    /// [`Self::iter_piece_faces_unfolded`], and `t` that walker's unfoldedness.
    /// The corners come back in the same piece-local unfolded space the walker
    /// works in, so placing them on a page still needs the piece transform.
    ///
    /// The anchor a flap leans on is the third vertex of the face *across* the
    /// cut, which is only in the right place once that face has been rotated onto
    /// this one — the neighbour belongs to a different piece (or to none), so its
    /// own unfolding says nothing about where it sits relative to this seam.
    pub fn piece_flap_corners(
        &self,
        l_id: LoopId,
        affine: Matrix4<f32>,
        t: f32,
    ) -> Option<[Point3<f32>; 4]> {
        if !self.loop_has_flap(l_id) {
            return None;
        }
        let l = self[l_id];
        let across = self[l.radial_next];
        let (v0_id, v1_id) = (self[l.e].v[0], self[l.e].v[1]);
        let (v0, v1) = (self.vert_pos(v0_id), self.vert_pos(v1_id));

        // Rotate the facing face onto this one about the shared edge, by the
        // signed angle between the two face normals.
        let axis = (v1 - v0).normalize();
        let (n_here, n_across) = (Vector3::from(self[l.f].no), Vector3::from(self[across.f].no));
        let angle = axis.dot(n_across.cross(n_here)).atan2(n_across.dot(n_here)) * t;
        let local = Matrix4::from_translation(v0)
            * Matrix4::from_axis_angle(axis, Rad(angle))
            * Matrix4::from_translation(-v0);

        // The anchor is the facing face's one vertex that isn't on the seam.
        let anchor = self
            .iter_face_loops(across.f)
            .map(|l| self[l].v)
            .find(|v| *v != v0_id && *v != v1_id)?;
        let anchor = (affine * local).transform_point(Point3::from_vec(self.vert_pos(anchor)));

        Some(flap_corners(
            affine.transform_point(Point3::from_vec(v0)).to_vec(),
            affine.transform_point(Point3::from_vec(v1)).to_vec(),
            anchor.to_vec(),
        ))
    }

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
mod corner_tests {
    use cgmath::{InnerSpace, Vector3};

    use super::{flap_corners, MAX_FLAP_APEX_ANGLE, MAX_FLAP_HEIGHT};

    /// A base edge along x, with the anchor `reach` away from it in +y — the
    /// unfolded layout every flap sees, up to a rigid transform.
    fn flap(base_len: f32, reach: f32) -> [Vector3<f32>; 3] {
        [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(base_len, 0.0, 0.0),
            Vector3::new(0.5 * base_len, reach, 0.0),
        ]
    }

    /// The trapezoid stands on the base edge itself, so the two share their
    /// endpoints exactly. Gluing depends on it: the base is the fold line.
    #[test]
    fn the_flap_stands_on_its_base_edge() {
        let [v0, v1, v2] = flap(1.0, 0.4);
        let corners = flap_corners(v0, v1, v2);
        assert_eq!(corners[0], cgmath::Point3::new(0.0, 0.0, 0.0));
        assert_eq!(corners[1], cgmath::Point3::new(1.0, 0.0, 0.0));
    }

    /// The flap leans toward the anchor, never away from it — a tab folded the
    /// wrong way lands outside the face it is supposed to be glued under.
    #[test]
    fn the_flap_leans_toward_the_anchor() {
        let [v0, v1, v2] = flap(1.0, 0.4);
        let corners = flap_corners(v0, v1, v2);
        assert!(corners[2].y > 0.0, "top-right should lean toward the anchor");
        assert!(corners[3].y > 0.0, "top-left should lean toward the anchor");

        // Mirror the anchor and the flap must follow it.
        let mirrored = flap_corners(v0, v1, Vector3::new(0.5, -0.4, 0.0));
        assert!(mirrored[2].y < 0.0 && mirrored[3].y < 0.0);
    }

    /// A symmetric neighbour gives a symmetric tab. Asymmetry here would show up
    /// as a tab that visibly slews to one side of the seam.
    #[test]
    fn a_symmetric_anchor_gives_a_symmetric_flap() {
        let [v0, v1, v2] = flap(2.0, 0.5);
        let corners = flap_corners(v0, v1, v2);
        let mid = 0.5 * (corners[0].x + corners[1].x);
        assert!(
            ((corners[3].x - mid) + (corners[2].x - mid)).abs() < 1e-5,
            "top corners should straddle the base midpoint: {corners:?}"
        );
        assert!((corners[2].y - corners[3].y).abs() < 1e-6, "and sit at one height");
    }

    /// Height is capped, so a long seam with a lot of room still gets a tab you
    /// can glue rather than one that eats the page.
    #[test]
    fn the_flap_height_is_capped() {
        // A base this long would inscribe a 45-degree triangle 5cm tall.
        let [v0, v1, v2] = flap(10.0, 20.0);
        let corners = flap_corners(v0, v1, v2);
        assert!(
            corners[2].y <= MAX_FLAP_HEIGHT + 1e-6,
            "expected the cap at {MAX_FLAP_HEIGHT}, got {}",
            corners[2].y
        );
    }

    /// A shallow neighbour caps the tab below the height limit instead: the tab
    /// has to stay inside the face it folds onto.
    #[test]
    fn a_shallow_anchor_shortens_the_flap_below_the_cap() {
        let [v0, v1, v2] = flap(1.0, 0.05);
        let corners = flap_corners(v0, v1, v2);
        assert!(
            corners[2].y < MAX_FLAP_HEIGHT,
            "a shallow face should bound the flap before the height cap does"
        );
        assert!(corners[2].y > 0.0, "but it should still exist");
    }

    /// The inscribed triangle's apex angle never exceeds 45 degrees, whatever the
    /// facing face's shape, so a very obtuse neighbour can't grow a spike.
    #[test]
    fn the_apex_angle_never_exceeds_the_cap() {
        for reach in [0.05, 0.2, 1.0, 5.0, 100.0] {
            let [v0, v1, v2] = flap(1.0, reach);
            let corners = flap_corners(v0, v1, v2);
            // Re-derive the untruncated apex from the trapezoid's own sides.
            let side = corners[3] - corners[0];
            let base = (corners[1] - corners[0]).normalize();
            let angle = side.normalize().dot(base).clamp(-1.0, 1.0).acos();
            assert!(
                angle <= MAX_FLAP_APEX_ANGLE + 1e-5,
                "reach {reach} gave a base angle of {angle} rad, over the cap"
            );
        }
    }

    /// An anchor collinear with the base gives the flap no direction to lean, so
    /// it collapses instead of producing NaN corners the GPU would scatter.
    #[test]
    fn a_degenerate_anchor_collapses_the_flap() {
        let v0 = Vector3::new(0.0, 0.0, 0.0);
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        for anchor in [Vector3::new(0.5, 0.0, 0.0), Vector3::new(2.0, 0.0, 0.0), v0] {
            let corners = flap_corners(v0, v1, anchor);
            assert!(
                corners.iter().all(|c| c.x.is_finite() && c.y.is_finite() && c.z.is_finite()),
                "collinear anchor {anchor:?} produced non-finite corners: {corners:?}"
            );
            assert_eq!(corners[3], corners[0], "and the tab should have no height");
            assert_eq!(corners[2], corners[1]);
        }
    }

    /// A zero-length base has no flap to build, and must not divide by it.
    #[test]
    fn a_zero_length_base_collapses_the_flap() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let corners = flap_corners(v, v, Vector3::new(1.0, 3.0, 3.0));
        assert!(corners.iter().all(|c| c.x.is_finite() && c.y.is_finite() && c.z.is_finite()));
    }

    /// The flap is built in the unfolded plane the three points span, so it works
    /// just as well on a piece lying off the page's own plane.
    #[test]
    fn the_flap_follows_the_plane_its_points_span() {
        // Same triangle as `flap(1.0, 0.4)`, rotated into the xz plane.
        let corners = flap_corners(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.5, 0.0, 0.4),
        );
        let flat = flap_corners(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.5, 0.4, 0.0),
        );
        assert!((corners[3].z.abs() - flat[3].y.abs()).abs() < 1e-6);
        assert!(corners[3].y.abs() < 1e-6, "the flap should stay in its own plane");
    }
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
