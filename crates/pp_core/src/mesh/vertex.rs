use crate::id::{self, Id};

use super::MeshElementType;

/// A single vertex in space.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// Vertex positions
    pub po: [f32; 3],
    /// Vertex normals
    pub no: [f32; 3],

    /// DiskCycle: Any edge containing this vertex
    pub e: Option<id::EdgeId>,
    /// The "index" of this vertex in final VBO, not accounting for face-data
    pub index: Option<usize>,
}

impl super::Mesh {
    /// Adds a single, disconnected vertex to the mesh.
    pub fn add_vertex(&mut self, po: [f32; 3], no: [f32; 3]) -> id::VertexId {
        self.elem_dirty |= MeshElementType::VERTS;
        id::VertexId::from_usize(self.verts.push(Vertex { e: None, po, no, index: None }))
    }
}

// --- Section: Disk Cycle ---

/// Enables walking the edges around a vertex
pub struct DiskCycleWalker<'mesh> {
    mesh: &'mesh super::Mesh,
    v: id::VertexId,
    e_start: id::EdgeId,
    e_curr: id::EdgeId,
    done: bool,
}

impl<'mesh> DiskCycleWalker<'mesh> {
    pub fn new(mesh: &'mesh super::Mesh, e_start: id::EdgeId, v: id::VertexId) -> Self {
        Self { mesh, v, e_start, e_curr: e_start, done: false }
    }
}

impl Iterator for DiskCycleWalker<'_> {
    type Item = id::EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let e = self.e_curr;
        self.e_curr = self.mesh[e].disklink(self.v).next;
        self.done = self.e_curr == self.e_start;
        Some(e)
    }
}

impl DoubleEndedIterator for DiskCycleWalker<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let e = self.e_curr;
        self.e_curr = self.mesh[e].disklink(self.v).prev;
        self.done = self.e_curr == self.e_start;
        Some(e)
    }
}

impl super::Mesh {
    /// Walks the edges around a vertex. You must pass the ID of an edge to
    /// start the walk with.
    pub fn iter_vert_edges(&self, e: id::EdgeId, v: id::VertexId) -> DiskCycleWalker {
        DiskCycleWalker::new(self, e, v)
    }
}
