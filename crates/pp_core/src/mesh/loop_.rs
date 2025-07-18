use crate::id::{self, Id};

/// A loop, best thought of as a "corner" of a face. Corresponds to exactly
/// one face, vertex, and edge.
#[derive(Debug, Clone, Copy, Default)]
pub struct Loop {
    pub v: id::VertexId,
    pub e: id::EdgeId,
    pub f: id::FaceId,

    // UVs and normals are per-vertex per-face
    pub uv: [f32; 2],
    pub no: [f32; 3],

    // RadialCycle: Other loops around the edge
    pub radial_next: id::LoopId,
    pub radial_prev: id::LoopId,

    // LoopCycle: Other loops in this face
    pub next: id::LoopId,
    pub prev: id::LoopId,
}

impl super::Mesh {
    /// Iterates all the loops in the mesh, in pre-defined order. Most commonly
    /// used to build up VBOs (without IBOs) by `pp_draw`.
    pub fn iter_loops(&self) -> impl Iterator<Item = id::LoopId> + '_ {
        self.faces.indices().flat_map(|f_id| self.iter_face_loops(id::FaceId::from_usize(f_id)))
    }

    /// Adds a loop into the radial loop cycle around an edge.
    pub(super) fn connect_loop_to_edge(&mut self, l: id::LoopId, e: id::EdgeId) {
        // If the edge already has a loop, update that loop's Radial links
        if let Some(l_first) = self[e].l {
            let l_next = (&*self)[l_first].radial_next;
            self[l].radial_prev = l_first;
            self[l].radial_next = l_next;
            self[l_next].radial_prev = l;
            self[l_first].radial_next = l;
            self[e].l = Some(l);
        } else {
            // Otherwise, this edge has no face, so Radial points to itself
            self[l].radial_prev = l;
            self[l].radial_next = l;
            self[e].l = Some(l);
        }
        // Point the loop back at the edge itself
        self[l].e = e;
    }
}
