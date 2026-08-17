use cgmath::{EuclideanSpace, InnerSpace, Point3, Transform, Vector3};

use crate::State;

/// An axis-aligned bounding box in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3 {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb3 {
    pub const EMPTY: Self = Self {
        min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x
    }

    /// Grows the box to include the given point.
    pub fn extend(&mut self, p: Vector3<f32>) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Returns the smallest box containing both `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        Self {
            min: Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn size(&self) -> Vector3<f32> {
        if self.is_empty() {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        self.max - self.min
    }

    /// The midpoint of the box, or the origin if it is empty.
    pub fn center(&self) -> Vector3<f32> {
        if self.is_empty() {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        (self.min + self.max) / 2.0
    }

    /// Half the diagonal of the box: the radius of the smallest sphere
    /// guaranteed to contain it, regardless of viewing angle. Used to size
    /// camera zoom/dolly limits so the whole box stays reachable.
    pub fn bounding_radius(&self) -> f32 {
        self.size().magnitude() / 2.0
    }

    /// The 8 corners of the box, in a fixed order shared by [`Self::EDGES`].
    pub fn corners(&self) -> [Vector3<f32>; 8] {
        [
            Vector3::new(self.min.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.max.y, self.max.z),
            Vector3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Corner-index pairs (into [`Self::corners`]) forming the box's 12 edges.
    pub const EDGES: [(u8, u8); 12] = [
        // bottom face
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // top face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // verticals connecting bottom to top
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
}

/// Bounds of the current selection, used to frame it in the viewports.
///
/// The two viewports draw the document in different spaces, so each needs its
/// own box: the folding viewport draws the folded mesh (`vbo::pos` plus the
/// mesh's model transform), while the cutting viewport draws the unfolded
/// pieces (`vbo::piece_pos` plus the piece's transform).
impl State {
    /// Whether the selection has anything the caller cares about.
    ///
    /// `faces_only` narrows this to faces, so leftovers from an earlier
    /// vertex- or edge-mode selection don't count as a selection while the
    /// user is working with faces or pieces.
    fn selection_is_empty(&self, faces_only: bool) -> bool {
        self.selection.faces.is_empty()
            && (faces_only || (self.selection.verts.is_empty() && self.selection.edges.is_empty()))
    }

    /// The folded, world-space bounds of the selection, i.e. what the folding
    /// viewport draws. Falls back to [`Self::world_bounds`] when nothing is
    /// selected.
    ///
    /// Selecting a face propagates to its edges but *not* to its verts (see
    /// [`Self::select_face`]), so all three selection sets have to be unioned
    /// to cover vertex and edge modes. `faces_only` drops the vert and edge
    /// sets entirely, for the face and piece modes where those elements aren't
    /// what the user is selecting.
    pub fn selection_bounds(&self, faces_only: bool) -> Aabb3 {
        if self.selection_is_empty(faces_only) {
            return self.world_bounds();
        }
        let mut aabb = Aabb3::EMPTY;
        let mut extend = |m_id, v_id| {
            let mesh = &self.meshes[m_id];
            let p = mesh.transform.transform_point(Point3::from_vec(mesh.vert_pos(v_id)));
            aabb.extend(p.to_vec());
        };
        if !faces_only {
            self.selection.verts.iter().for_each(|(m_id, v_id)| extend(*m_id, *v_id));
            self.selection.edges.iter().for_each(|(m_id, e_id)| {
                self.meshes[*m_id][*e_id].v.iter().for_each(|v_id| extend(*m_id, *v_id))
            });
        }
        self.selection.faces.iter().for_each(|(m_id, f_id)| {
            let mesh = &self.meshes[*m_id];
            mesh.iter_face_loops(*f_id).for_each(|l| extend(*m_id, mesh[l].v))
        });
        aabb
    }

    /// The unfolded, piece-transformed bounds of the selection, i.e. what the
    /// cutting viewport draws. Falls back to every piece when nothing is
    /// selected.
    ///
    /// This walks all pieces rather than deriving the pieces from the
    /// selection, so vertex-, edge-, face-, and piece-mode selections are all
    /// handled by the same pass. It costs one traversal of geometry the draw
    /// cache already rebuilds every frame, which is fine at keypress rate.
    ///
    /// `faces_only` has the same meaning as in [`Self::selection_bounds`].
    pub fn selection_piece_bounds(&self, faces_only: bool) -> Aabb3 {
        let select_all = self.selection_is_empty(faces_only);
        let mut aabb = Aabb3::EMPTY;
        self.meshes.iter().for_each(|(m_id, mesh)| {
            mesh.iter_pieces().for_each(|root| {
                let Some(piece) = mesh.pieces.get(root) else { return };
                mesh.iter_piece_faces_unfolded(*root).for_each(|face| {
                    let face_selected = self.selection.faces.contains(&(m_id, face.f));
                    mesh.iter_face_loops(face.f).for_each(|l| {
                        let l = mesh[l];
                        let loose_selected = !faces_only
                            && (self.selection.edges.contains(&(m_id, l.e))
                                || self.selection.verts.contains(&(m_id, l.v)));
                        if !select_all && !face_selected && !loose_selected {
                            return;
                        }
                        let p = piece.transform.transform_point(
                            face.affine.transform_point(Point3::from_vec(mesh.vert_pos(l.v))),
                        );
                        aabb.extend(p.to_vec());
                    })
                })
            })
        });
        aabb
    }

    /// The normalized mean normal of the selection, i.e. the direction the 3D
    /// camera should look at it from. `None` when there's no meaningful
    /// facing: nothing selected, only loose verts selected, or normals that
    /// cancel each other out (e.g. a whole closed mesh).
    ///
    /// `faces_only` has the same meaning as in [`Self::selection_bounds`].
    pub fn selection_normal(&self, faces_only: bool) -> Option<Vector3<f32>> {
        let face_normal = |m_id: crate::MeshId, f_id: crate::id::FaceId| {
            let mesh = &self.meshes[m_id];
            mesh.transform.transform_vector(Vector3::from(mesh[f_id].no))
        };
        let sum = if faces_only || !self.selection.faces.is_empty() {
            self.selection.faces.iter().fold(Vector3::new(0.0, 0.0, 0.0), |acc, (m_id, f_id)| {
                acc + face_normal(*m_id, *f_id)
            })
        } else {
            // No faces selected: fall back to the faces the selected edges
            // border, so edge-mode selections still get a facing.
            self.selection.edges.iter().fold(Vector3::new(0.0, 0.0, 0.0), |acc, (m_id, e_id)| {
                let mesh = &self.meshes[*m_id];
                mesh.iter_edge_loops(*e_id)
                    .into_iter()
                    .flatten()
                    .fold(acc, |acc, l| acc + face_normal(*m_id, mesh[l].f))
            })
        };
        (sum.magnitude2() > 1e-6).then(|| sum.normalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{FaceId, Id, VertexId};
    use crate::select::SelectionActionType;

    /// The first face of the only mesh in a cube document.
    fn cube() -> (crate::State, crate::MeshId, FaceId) {
        let state = crate::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        (state, m_id, FaceId::from_usize(0))
    }

    /// Selecting a face propagates to its edges but never to its verts, so
    /// bounds built from `selection.verts` alone would come back empty.
    #[test]
    fn a_face_only_selection_still_has_bounds() {
        let (mut state, m_id, f_id) = cube();
        state.select_face(&(m_id, f_id), SelectionActionType::Select, false, true);
        assert!(state.selection.verts.is_empty(), "test premise: no verts get selected");

        let aabb = state.selection_bounds(false);
        assert!(!aabb.is_empty());
        // The cube's first face is a triangle of the z=0 bottom
        assert_eq!(aabb.min, Vector3::new(-0.5, -0.5, 0.0));
        assert_eq!(aabb.max, Vector3::new(0.5, 0.5, 0.0));
    }

    #[test]
    fn a_selected_face_reports_its_own_normal() {
        let (mut state, m_id, f_id) = cube();
        state.select_face(&(m_id, f_id), SelectionActionType::Select, false, true);
        let expected = Vector3::from(state.meshes[m_id][f_id].no).normalize();
        let normal = state.selection_normal(false).unwrap();
        assert!((normal - expected).magnitude() < 1e-5, "{normal:?} should be {expected:?}");
    }

    /// A closed mesh's normals cancel out, leaving no meaningful direction to
    /// look at it from.
    #[test]
    fn a_whole_closed_mesh_has_no_selection_normal() {
        let (mut state, _, _) = cube();
        state.select_all(SelectionActionType::Select);
        assert!(state.selection_normal(false).is_none());
    }

    #[test]
    fn an_empty_selection_falls_back_to_the_whole_document() {
        let (state, _, _) = cube();
        assert!(state.selection.faces.is_empty());
        assert_eq!(state.selection_bounds(false), state.world_bounds());
    }

    /// The case `faces_only` exists for: switching to face mode leaves the
    /// verts and edges of an earlier selection behind, and those shouldn't
    /// drag the frame towards geometry the user isn't selecting any more.
    #[test]
    fn faces_only_ignores_leftover_verts_and_edges() {
        let (mut state, m_id, _) = cube();
        // A side face, plus a stray vert on the far corner of the cube
        let f_id = FaceId::from_usize(4);
        state.select_face(&(m_id, f_id), SelectionActionType::Select, false, true);
        let far = state.meshes[m_id].verts.indices().map(VertexId::from_usize).max_by(|a, b| {
            let key = |v| state.meshes[m_id].vert_pos(v).x;
            key(*a).partial_cmp(&key(*b)).unwrap()
        });
        state.select_vert(&(m_id, far.unwrap()), SelectionActionType::Select, false);

        let with_loose = state.selection_bounds(false);
        let faces_only = state.selection_bounds(true);
        assert!(
            with_loose.size().x > faces_only.size().x,
            "the stray vert should stretch the all-elements bounds ({with_loose:?}) \
             past the faces-only ones ({faces_only:?})"
        );
        // The faces-only box is exactly the face: a quad on the x = -0.5 plane
        assert_eq!(faces_only.size().x, 0.0);
    }

    /// Face mode with only stray verts left over counts as no selection, so
    /// framing falls back to the whole document rather than to those verts.
    #[test]
    fn faces_only_with_no_faces_falls_back_to_the_whole_document() {
        let (mut state, m_id, _) = cube();
        let v_id = state.meshes[m_id].verts.indices().map(VertexId::from_usize).next().unwrap();
        state.select_vert(&(m_id, v_id), SelectionActionType::Select, false);
        assert!(state.selection.faces.is_empty(), "test premise: one vert selects no faces");

        assert_eq!(state.selection_bounds(true), state.world_bounds());
        assert_ne!(state.selection_bounds(false), state.world_bounds());
        assert!(state.selection_normal(true).is_none());
    }

    #[test]
    fn piece_bounds_follow_the_piece_transform() {
        let mut state = crate::State::default();
        // A lone triangle: one face, so it makes a piece without any cuts
        let m_id = state.meshes.insert(crate::mesh::Mesh::new_tri());
        let f_id = FaceId::from_usize(0);
        state.meshes[m_id].expand_piece(f_id).unwrap();

        let before = state.selection_piece_bounds(false);
        assert!(!before.is_empty(), "an unselected document frames all of its pieces");

        let shift = Vector3::new(3.0, -2.0, 0.0);
        state.meshes[m_id].transform_piece(&f_id, cgmath::Matrix4::from_translation(shift));
        let after = state.selection_piece_bounds(false);
        assert!(
            (after.min - (before.min + shift)).magnitude() < 1e-5
                && (after.max - (before.max + shift)).magnitude() < 1e-5,
            "bounds {after:?} should be {before:?} shifted by {shift:?}"
        );
    }

    #[test]
    fn piece_bounds_narrow_to_the_selection() {
        let mut state = crate::State::default();
        let m_id = state.meshes.insert(crate::mesh::Mesh::new_tri());
        let f_id = FaceId::from_usize(0);
        state.meshes[m_id].expand_piece(f_id).unwrap();
        let all = state.selection_piece_bounds(false);

        // Selecting a single vert of the piece collapses the box onto it
        let v_id = state.meshes[m_id][state.meshes[m_id][f_id].l].v;
        state.select_vert(&(m_id, v_id), SelectionActionType::Select, false);
        let one = state.selection_piece_bounds(false);
        assert_eq!(one.size(), Vector3::new(0.0, 0.0, 0.0));
        assert!(one.min.x >= all.min.x - 1e-5 && one.max.x <= all.max.x + 1e-5);
    }
}
