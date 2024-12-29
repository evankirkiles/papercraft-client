use bitflags::bitflags;
use std::ops;
use walk_adj::{DiskCycleWalker, LoopCycleWalker, RadialCycleWalker};

mod primitives;
mod walk_adj;

use crate::id::{EdgeId, FaceId, Id, LoopId, MeshId, VertexId};
use stable_vec::StableVec;

bitflags! {
    pub struct MeshDirtyFlags: u8 {
        const VERTS = 1 << 0;
        const EDGES = 1 << 1;
        const FACES = 1 << 2;
        const LOOPS = 1 << 3;
    }
}

/// A topology-enabled Mesh
///
/// Uses Blender's BMesh topological mesh representation for flexibility in
/// handling potentially non-manifold meshes.
///
/// Set up CPU / topology resources:
///  - Create all vertices
///  - Create all faces between vertices
///
/// Set up GPU resources:
///  - Use "loop"s to build VBOs (duplicate vertices per face)
///  - Use "faces.mat_nr" to buld IBOs
///
/// @see https://developer.blender.org/docs/features/objects/mesh/bmesh/
pub struct Mesh {
    pub id: MeshId,
    pub label: String,

    pub verts: StableVec<Vertex>,
    pub edges: StableVec<Edge>,
    pub faces: StableVec<Face>,
    pub loops: StableVec<Loop>,

    /// Indicates which type of element has changed in this mesh
    pub elem_dirty: MeshDirtyFlags,
    pub index_dirty: MeshDirtyFlags,
}

/// A single vertex in space.
#[derive(Clone, Copy)]
pub struct Vertex {
    /// Vertex positions
    pub po: [f32; 3],
    /// Vertex normals
    pub no: [f32; 3],

    /// DiskCycle: Any edge containing this vertex
    pub e: Option<EdgeId>,
}

/// An edge, formed by two vertices.
#[derive(Clone, Copy)]
pub struct Edge {
    /// Vertices connected by this edge
    pub v: [VertexId; 2],
    /// DiskCycle: Support radially iterating the edges of each vertex
    pub dl: [DiskLink; 2],

    /// RadialCycle: Any loop (defined by a face) for this specific edge
    pub l: Option<LoopId>,
}

impl Edge {
    /// Creates a new Edge with DiskLinks referencing just itself
    pub fn new(e: EdgeId, v1: VertexId, v2: VertexId) -> Self {
        Self { v: [v1, v2], dl: [DiskLink::new(e), DiskLink::new(e)], l: None }
    }

    /// Ensures that this edge contains vertex `v`
    pub fn has_vert(&self, v: VertexId) -> bool {
        self.v[0] == v || self.v[1] == v
    }

    /// Gets an immutable reference to the DiskLink for a specific vertex
    pub fn disklink(&self, v: VertexId) -> &DiskLink {
        assert!(self.has_vert(v));
        &self.dl[if self.v[0] == v { 0 } else { 1 }]
    }

    /// Gets a mutable reference to the DiskLink for a specific vertex
    pub fn disklink_mut(&mut self, v: VertexId) -> &mut DiskLink {
        assert!(self.has_vert(v));
        &mut self.dl[if self.v[0] == v { 0 } else { 1 }]
    }
}

/// A face, formed by three vertices and three edges.
#[derive(Clone, Copy)]
pub struct Face {
    /// Face normal
    pub no: [f32; 3],
    /// Material index for this face
    pub mat_nr: u16,
    /// The number of vertices of this face. This will always be 3
    pub len: usize,

    /// LoopCycle: Any loop in this face
    pub l_first: LoopId,
}

impl Face {
    /// Creates a new Face with a temporary Loop Id
    fn new(len: usize) -> Self {
        Self { no: [0.0, 0.0, 0.0], mat_nr: 0, len, l_first: LoopId::temp() }
    }
}

/// A disk link for quick iteration of edges around a vertex
#[derive(Clone, Copy)]
pub struct DiskLink {
    pub prev: EdgeId,
    pub next: EdgeId,
}

impl DiskLink {
    /// Creates a new DiskLink referencing just the single edge
    pub fn new(e: EdgeId) -> Self {
        Self { prev: e, next: e }
    }
}

/// A loop, best thought of as a "corner" of a face. Corresponds to exactly
/// one face, vertex, and edge.
#[derive(Clone, Copy)]
pub struct Loop {
    pub v: VertexId,
    pub e: EdgeId,
    pub f: FaceId,

    // RadialCycle: Other loops around the edge
    pub radial_next: LoopId,
    pub radial_prev: LoopId,

    // LoopCycle: Other loops in this face
    pub next: LoopId,
    pub prev: LoopId,
}

impl Loop {
    /// Creates a new Loop with temporary radial / loop links. You *must*
    /// set the radial / loop links once the face is fully created, otherwise
    /// you'll run into all sorts of adjacency query issues.
    pub fn new(f: FaceId, v: VertexId, e: EdgeId) -> Self {
        Self {
            v,
            e,
            f,
            next: LoopId::temp(),
            prev: LoopId::temp(),
            radial_next: LoopId::temp(),
            radial_prev: LoopId::temp(),
        }
    }
}

impl Mesh {
    pub fn new(id: MeshId, label: String) -> Self {
        Self {
            id,
            label,
            verts: StableVec::new(),
            edges: StableVec::new(),
            faces: StableVec::new(),
            loops: StableVec::new(),
            elem_dirty: MeshDirtyFlags::empty(),
            index_dirty: MeshDirtyFlags::empty(),
        }
    }

    // --- Section: Mesh Construction ---

    /// Adds a single, disconnected vertex to the mesh.
    pub fn add_vertex(&mut self, po: [f32; 3], no: [f32; 3]) -> VertexId {
        self.elem_dirty |= MeshDirtyFlags::VERTS;
        VertexId::from_usize(self.verts.push(Vertex { e: None, po, no }))
    }

    /// Adds an edge between two vertices.
    ///
    /// If an edge already existed between `v1` and `v2`, returns that instead.
    fn add_edge(&mut self, v1: VertexId, v2: VertexId) -> EdgeId {
        // If edge already exists, return it
        if let Some(e) = self.query_edge(v1, v2) {
            return e;
        }

        // Otherwise, create new edge, inserting it into the disks for each vertex
        let e = EdgeId::from_usize(self.edges.next_push_index());
        self.edges.push(Edge::new(e, v1, v2));
        self.disk_edge_append(e, v1);
        self.disk_edge_append(e, v2);

        // Mark edge resources as needing to be recreated
        self.elem_dirty |= MeshDirtyFlags::EDGES;
        self.index_dirty |= MeshDirtyFlags::EDGES;
        e
    }

    /// Adds an ngon / face between any number of vertices
    ///
    /// If a face already existed between the verts, returns that instead.
    pub fn add_face(&mut self, verts: &[VertexId]) -> FaceId {
        // If face already exists, return it
        if let Some(f_id) = self.query_face(verts) {
            return f_id;
        }

        // Otherwise, begin creating the face
        let len = verts.len();
        let f = FaceId::from_usize(self.faces.push(Face::new(len)));

        // Create or use existing edges between all adjacent vertices
        let edges: Vec<EdgeId> =
            (0..len).map(|i| self.add_edge(verts[i], verts[(i + 1) % len])).collect();

        // Create the loops for the face, adding loop + radial data
        let l_start = LoopId::from_usize(self.loops.push(Loop::new(f, verts[0], edges[0])));
        self.radial_loop_append(l_start, edges[0]);
        self[f].l_first = l_start;
        let mut l_last = l_start;
        for i in 1..len {
            let l = LoopId::from_usize(self.loops.push(Loop::new(f, verts[i], edges[i])));
            self.radial_loop_append(l, edges[i]);
            self[l].prev = l_last;
            self[l_last].next = l;
            l_last = l;
        }
        self[l_start].prev = l_last;
        self[l_last].next = l_start;

        // Face and loop resources need to be recreated
        self.elem_dirty |= MeshDirtyFlags::LOOPS | MeshDirtyFlags::FACES;
        self.index_dirty |= MeshDirtyFlags::LOOPS | MeshDirtyFlags::FACES;
        f
    }

    // --- Section: Queries ---

    /// Returns the edge between the supplied vertices, or None.
    fn query_edge(&self, v_a: VertexId, v_b: VertexId) -> Option<EdgeId> {
        if v_a == v_b {
            return None;
        }
        let (Some(e_a), Some(e_b)) = (self[v_a].e, self[v_b].e) else {
            return None;
        };
        let (mut e_a_iter, mut e_b_iter) = (e_a, e_b);
        loop {
            if self[e_a_iter].has_vert(v_b) {
                return Some(e_a_iter);
            }
            if self[e_b_iter].has_vert(v_a) {
                return Some(e_b_iter);
            }
            e_a_iter = self[e_a_iter].disklink(v_a).next;
            e_b_iter = self[e_b_iter].disklink(v_b).next;
            // If we made a full loop, terminate
            if e_a_iter == e_a || e_b_iter == e_b {
                break;
            }
        }
        None
    }

    /// Returns the face between the supplied vertices, or None.
    ///
    /// TODO: How do we ensure a consistent vertex order?
    fn query_face(&self, verts: &[VertexId]) -> Option<FaceId> {
        let len = verts.len();
        let v0 = verts[0];
        let e0 = self[v0].e?;
        let (mut e_iter, e_first) = (e0, e0);
        loop {
            // Cycle 1: Disk on v0, aka edges around v0
            if let Some(l) = self[e_iter].l {
                let (mut l_iter_radial, l_first_radial) = (l, l);
                loop {
                    // Cycle 2: Loops (radial) for each edge, aka faces containing e_iter
                    let l_curr = self[l_iter_radial];
                    if l_curr.v == v0 && self[l_curr.f].len == len {
                        // First two verts match, so iterate through for remaining verts
                        // Note that loop winding direction is undefined, so we
                        // need to iterate in both directions (next's and prev's).
                        let mut i_walk = 2;
                        // Cycle 3a: Loops in face, forwards
                        if self[l_curr.next].v == verts[1] {
                            let mut l_walk = self[l_curr.next].next;
                            loop {
                                if self[l_walk].v != verts[i_walk] {
                                    break;
                                }
                                l_walk = self[l_walk].next;
                                i_walk += 1;
                                if i_walk == len {
                                    break;
                                }
                            }
                        // Cycle 3b: Loops in face, backwards
                        } else if self[l_curr.prev].v == verts[1] {
                            let mut l_walk = self[l_curr.prev].prev;
                            loop {
                                if self[l_walk].v != verts[i_walk] {
                                    break;
                                }
                                l_walk = self[l_walk].prev;
                                i_walk += 1;
                                if i_walk == len {
                                    break;
                                }
                            }
                        }

                        // If there's a loop for every vertex, face matches
                        if i_walk == len {
                            return Some(l_curr.f);
                        }
                    }

                    // LoopCycle: Go to next loop around e_iter
                    l_iter_radial = self[l_iter_radial].radial_next;
                    if l_iter_radial == l_first_radial {
                        break;
                    }
                }
            }
            // DiskCycle: Go to next disk edge around v0
            e_iter = self[e_iter].disklink(v0).next;
            if e_iter == e_first {
                break;
            };
        }
        None
    }

    // --- Section: "Disk" Cycle ---

    /// Adds an edge into the disk cycle around a vertex.
    fn disk_edge_append(&mut self, e: EdgeId, v: VertexId) {
        // If the vertex already has an edge, update that edge's DiskLink
        if let Some(e_first) = self[v].e {
            let e_last = self[e_first].disklink(v).prev;
            let dl = self[e].disklink_mut(v);
            dl.next = e_first;
            dl.prev = e_last;
            self[e_first].disklink_mut(v).prev = e;
            self[e_last].disklink_mut(v).next = e;
        } else {
            // Otherwise, this is an isolated vertex, so DiskLink points to itself
            let dl = self[e].disklink_mut(v);
            dl.next = e;
            dl.prev = e;
            self[v].e = Some(e);
        }
    }

    /// Walks the edges including a vertex (faces)
    pub fn disk_edge_walk(&self, e: EdgeId, v: VertexId) -> DiskCycleWalker {
        DiskCycleWalker::new(self, e, v)
    }

    // --- Section: "Radial" Cycle ---

    /// Adds a loop into the radial loop cycle around an edge
    fn radial_loop_append(&mut self, l: LoopId, e: EdgeId) {
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

    /// Walks the loops including an edge (faces)
    pub fn radial_loop_walk(&self, e: EdgeId) -> Option<RadialCycleWalker> {
        Some(RadialCycleWalker::new(self, self[e].l?))
    }

    // --- Section: "Loop" Cycle ---

    /// Walks the loops in a face (vertices)
    pub fn face_loop_walk(&self, f: FaceId) -> LoopCycleWalker {
        LoopCycleWalker::new(self, self[f].l_first)
    }
}

/// Automatically derive mutable and immutable indexing operations on the Mesh
/// struct for each of its ID'able element types.
macro_rules! impl_index {
    ($handle:ident, $field:ident, $out:ty) => {
        impl ops::Index<$handle> for Mesh {
            type Output = $out;

            #[inline(always)]
            fn index(&self, idx: $handle) -> &Self::Output {
                // &self.$field[*idx]
                unsafe { self.$field.get_unchecked(idx.to_usize()) }
            }
        }

        impl ops::IndexMut<$handle> for Mesh {
            #[inline(always)]
            fn index_mut(&mut self, idx: $handle) -> &mut Self::Output {
                // &mut self.$field[*idx]
                unsafe { self.$field.get_unchecked_mut(idx.to_usize()) }
            }
        }
    };
}

impl_index!(VertexId, verts, Vertex);
impl_index!(FaceId, faces, Face);
impl_index!(EdgeId, edges, Edge);
impl_index!(LoopId, loops, Loop);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_duplicate_edge() {
        let mut mesh = Mesh::new(MeshId::new(0), String::from("test"));
        let v1 = mesh.add_vertex([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let v2 = mesh.add_vertex([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        let e1 = mesh.add_edge(v1, v2);
        let e2 = mesh.add_edge(v2, v1);
        assert_eq!(e1, e2);
        assert_eq!(mesh.verts.num_elements(), 2);
        assert_eq!(mesh.edges.num_elements(), 1);
    }

    #[test]
    fn add_duplicate_face() {
        let mut mesh = Mesh::new(MeshId::new(0), String::from("test"));
        let v1 = mesh.add_vertex([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let v2 = mesh.add_vertex([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        let v3 = mesh.add_vertex([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
        let f1 = mesh.add_face(&[v1, v2, v3]);
        let f2 = mesh.add_face(&[v1, v3, v2]);
        assert_eq!(f1, f2);
        assert_eq!(mesh.verts.num_elements(), 3);
        assert_eq!(mesh.edges.num_elements(), 3);
        assert_eq!(mesh.loops.num_elements(), 3);
        assert_eq!(mesh.faces.num_elements(), 1);
    }
}
