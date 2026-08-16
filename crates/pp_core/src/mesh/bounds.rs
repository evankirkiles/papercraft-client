use cgmath::{EuclideanSpace, Point3, Transform, Vector3};

use crate::bounds::Aabb3;

impl super::Mesh {
    /// The mesh's local-space bounding box, post-scale but pre-transform.
    pub fn local_aabb(&self) -> Aabb3 {
        self.verts.values().fold(Aabb3::EMPTY, |mut acc, v| {
            acc.extend(Vector3::from(v.po) * self.scale);
            acc
        })
    }

    /// The mesh's world-space bounding box: the local AABB's 8 corners
    /// transformed by `self.transform` (translate + rotate) and re-bounded.
    /// This is a conservative AABB, not a tight oriented box, when the mesh
    /// is rotated.
    pub fn world_aabb(&self) -> Aabb3 {
        let local = self.local_aabb();
        if local.is_empty() {
            return local;
        }
        local.corners().into_iter().fold(Aabb3::EMPTY, |mut acc, c| {
            let p = self.transform.transform_point(Point3::from_vec(c));
            acc.extend(p.to_vec());
            acc
        })
    }
}
