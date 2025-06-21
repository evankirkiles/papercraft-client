use std::collections::{HashSet, VecDeque};

use cgmath::*;

use crate::{
    id::{self, Id, PieceId},
    mesh::MeshElementType,
};

/// A face, formed by three vertices and three edges.
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    /// Any face in the piece, used as the "root". "Unwrapping" the piece begins
    /// at this face and rotates faces along adjacent edges until all faces
    /// lie on the same plane.
    pub f: id::FaceId,

    /// In progressive unwrapping editors, this indicates the "unwrappedness"
    /// of the piece. t=0 means fully 3d, whereas t=1 means fully 2d
    pub t: f32,

    /// The transformation matrix of this piece
    pub transform: cgmath::Matrix4<f32>,

    /// Indicates if this piece's internal faces have changed
    pub elem_dirty: bool,
    /// Indicates if this piece's uniform data has changed, e.g. transform / hover state
    pub is_dirty: bool,
}

impl Piece {
    fn new(f: id::FaceId) -> Self {
        Self {
            f,
            t: 1.0,
            transform: cgmath::Matrix4::identity(),
            elem_dirty: true,
            is_dirty: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PieceCreationError {
    PieceAlreadyExists,
    CycleDetected,
}

impl super::Mesh {
    /// Tries to create a new piece from all the faces connected to a given face
    /// If you provide a `p_id`, then no new piece is created and all faces are
    /// assigned to the provided `p_id`.
    pub(crate) fn create_piece(
        &mut self,
        f_id: id::FaceId,
        p_id: Option<PieceId>,
    ) -> Result<id::PieceId, PieceCreationError> {
        self.assert_face_can_make_piece(f_id)?;
        let p_id = p_id.unwrap_or_else(|| {
            let piece = Piece::new(f_id);
            id::PieceId::from_usize(self.pieces.push(piece))
        });
        let f_ids: Vec<_> = self.iter_connected_faces(f_id).collect();
        f_ids.iter().for_each(|f_id| self[*f_id].p = Some(p_id));
        // Face and loop resources need to be recreated
        self.elem_dirty |= MeshElementType::PIECES;
        self.index_dirty |= MeshElementType::PIECES;
        Ok(p_id)
    }

    /// "Clears" a piece, returning all of its contained faces back to a no-piece state
    pub(crate) fn remove_piece(&mut self, p_id: id::PieceId, new_p_id: Option<id::PieceId>) {
        let f_id = self[p_id].f;
        let f_ids: Vec<_> = self.iter_connected_faces(f_id).collect();
        f_ids.iter().for_each(|f_id| self[*f_id].p = new_p_id);
        log::info!("Removed piece {p_id:?} for {new_p_id:?}");
        self.pieces.remove(p_id.to_usize());
        self.elem_dirty |= MeshElementType::PIECES;
        self.index_dirty |= MeshElementType::PIECES;
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
                    if self[e_id].cut.is_none() {
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

    /// Iterates all the loops in pieces of the mesh in pre-defined order.
    pub fn iter_piece_faces_unfolded(&self, p_id: id::PieceId) -> UnfoldedPieceFaceWalker {
        UnfoldedPieceFaceWalker::new(self, p_id)
    }

    /// Moves the piece, updating its transformation
    pub fn transform_piece(&mut self, p_id: id::PieceId, affine: cgmath::Matrix4<f32>) {
        self[p_id].transform = affine * self[p_id].transform;
        self[p_id].elem_dirty = true;
        self.elem_dirty |= MeshElementType::PIECES;
    }
}

// --- Section: Unfolded Piece Face Iterator ---

pub struct UnfoldedFace {
    pub f: id::FaceId,
    pub affine: Matrix4<f32>,
}

/// Walks over the loops within a piece, returning the true "unfolded" positions
/// of each vertex instead of just its static 3D position. The "unfolded"
/// position is computed based on the piece's 0-1 `t` value, indicating how
/// "unfolded" the piece should be.
pub struct UnfoldedPieceFaceWalker<'mesh> {
    mesh: &'mesh super::Mesh,
    /// 0-1, how much the pieces should be "unfolded"
    pub t: f32,
    /// The faces waiting to be explored, plus the affine transformation which
    /// all of their vertices must go through to be "unfolded"
    frontier: VecDeque<UnfoldedFace>,
    /// Faces already explored
    visited: HashSet<id::FaceId>,
    /// The final affine transformation to move vertices onto the XY plane
    pub affine_final: Matrix4<f32>,
}

impl<'mesh> UnfoldedPieceFaceWalker<'mesh> {
    fn new(mesh: &'mesh super::Mesh, p_id: id::PieceId) -> Self {
        let Piece { f, t, .. } = mesh[p_id];
        let up = Vector3::unit_z();
        let n = Vector3::from(mesh[f].no);

        // Get affine transform for entire piece onto XY plane at Z=0
        let axis = n.cross(up);
        let axis_len = axis.magnitude();
        let rotation = if axis_len < 1e-5 {
            // Normals are already aligned or opposite
            if n.dot(up) > 0.0 {
                Matrix4::identity()
            } else {
                // 180 degree rotation around any axis perpendicular to normal
                let arbitrary_axis = if n.x.abs() < 0.99 {
                    n.cross(Vector3::unit_x()).normalize()
                } else {
                    n.cross(Vector3::unit_y()).normalize()
                };
                Matrix4::from_axis_angle(arbitrary_axis, Rad(std::f32::consts::PI))
            }
        } else {
            let angle = n.angle(up);
            Matrix4::from_axis_angle(axis.normalize(), angle * t)
        };
        // 2. Translate point to lie on Z = 0
        let rotated_point = rotation.transform_vector(Vector3::from(mesh[mesh[mesh[f].l].v].po));
        let translation = Matrix4::from_translation(Vector3::new(0.0, 0.0, -rotated_point.z * t));
        let affine_final = translation * rotation;

        Self {
            mesh,
            t,
            affine_final,
            visited: HashSet::from([f]),
            frontier: VecDeque::from([UnfoldedFace { f, affine: Matrix4::identity() }]),
        }
    }
}

impl Iterator for UnfoldedPieceFaceWalker<'_> {
    type Item = UnfoldedFace;

    fn next(&mut self) -> Option<Self::Item> {
        let mut curr = self.frontier.pop_front()?;
        // Expand the frontier to include unvisited faces adjacent to this face
        self.frontier.extend(
            self.mesh
                .iter_face_loops(curr.f)
                .filter_map(|l| {
                    // Do not traverse across cut edges (keep within piece)
                    let e_id = self.mesh[l].e;
                    if self.mesh[e_id].cut.is_none() {
                        self.mesh.iter_edge_loops(e_id)
                    } else {
                        None
                    }
                })
                .flatten()
                // Compute affine matrix to rotate face into position given `t`
                .filter_map(|l_id| {
                    let l = self.mesh[l_id];
                    // Only visit unvisited faces
                    if !self.visited.insert(l.f) {
                        return None;
                    }

                    // 1. Get current positions of v0, v1 in untransformed space
                    // to determine the shared edge axis we need to rotate face 2 around
                    let v0 = Vector3::from(self.mesh[self.mesh[l.e].v[0]].po);
                    let v1 = Vector3::from(self.mesh[self.mesh[l.e].v[1]].po);
                    let axis = (v1 - v0).normalize();

                    // 2. Compare face normals to determine the angle we need to rotate face 2
                    // by (around the shared edge) to make it coplanar with face 1
                    // TODO: Use `t` to interpolate angle
                    let n1 = Vector3::from(self.mesh[curr.f].no);
                    let n2 = Vector3::from(self.mesh[l.f].no);
                    // Compute angle to rotate n2 onto n1 around `axis`
                    let cross = n2.cross(n1); // direction from n2 to n1
                    let dot = n2.dot(n1);
                    let angle = axis.dot(cross).atan2(dot) * self.t; // signed angle from n2 to n1

                    // 3. Create affine transform to rotate all vertices in face 2
                    // by `angle` around the shared edge, bringing face 2 onto the same
                    // plane to the "root" face.
                    let translation_origin = Matrix4::from_translation(-v0);
                    let rotation = Matrix4::from_axis_angle(axis, Rad(angle));
                    let translation_back = Matrix4::from_translation(v0);
                    let local_rotation = translation_back * rotation * translation_origin;

                    Some(UnfoldedFace { f: l.f, affine: curr.affine * local_rotation })
                }),
        );
        curr.affine = self.affine_final * curr.affine;
        Some(curr)
    }
}
