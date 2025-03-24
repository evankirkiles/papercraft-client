use crate::id::{self, Id};

/// A disk link for quick iteration of edges around a vertex
#[derive(Debug, Clone, Copy)]
pub struct DiskLink {
    pub prev: id::EdgeId,
    pub next: id::EdgeId,
}

impl DiskLink {
    /// Creates a new DiskLink referencing just the single edge
    pub fn new(e: id::EdgeId) -> Self {
        Self { prev: e, next: e }
    }
}

/// An edge, formed by two vertices.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    /// Vertices connected by this edge
    pub v: [id::VertexId; 2],
    /// DiskCycle: Support radially iterating the edges of each vertex
    pub dl: [DiskLink; 2],

    /// RadialCycle: Any loop (defined by a face) for this specific edge
    pub l: Option<id::LoopId>,
    /// The "index" of this edge in any final IBO
    pub index: Option<usize>,

    /// Is this edge cut or not?
    pub is_cut: bool,
}

impl Edge {
    /// Creates a new Edge with DiskLinks referencing just itself
    pub fn new(e: id::EdgeId, v1: id::VertexId, v2: id::VertexId) -> Self {
        Self {
            v: [v1, v2],
            dl: [DiskLink::new(e), DiskLink::new(e)],
            l: None,
            index: None,
            is_cut: false,
        }
    }

    /// Ensures that this edge contains vertex `v`
    pub fn has_vert(&self, v: id::VertexId) -> bool {
        self.v[0] == v || self.v[1] == v
    }

    /// Gets an immutable reference to the DiskLink for a specific vertex
    pub fn disklink(&self, v: id::VertexId) -> &DiskLink {
        assert!(self.has_vert(v));
        &self.dl[if self.v[0] == v { 0 } else { 1 }]
    }

    /// Gets a mutable reference to the DiskLink for a specific vertex
    pub fn disklink_mut(&mut self, v: id::VertexId) -> &mut DiskLink {
        assert!(self.has_vert(v));
        &mut self.dl[if self.v[0] == v { 0 } else { 1 }]
    }
}

impl super::Mesh {
    /// Adds an edge between two vertices. If an edge already existed between `v1` and `v2`,
    /// returns that instead.
    pub fn add_edge(&mut self, v1: id::VertexId, v2: id::VertexId) -> id::EdgeId {
        // If edge already exists, return it
        if let Some(e) = self.query_edge(v1, v2) {
            return e;
        }

        // Otherwise, create new edge, inserting it into the disks for each vertex
        let e = id::EdgeId::from_usize(self.edges.next_push_index());
        self.edges.push(Edge::new(e, v1, v2));
        self.connect_edge_to_vert(e, v1);
        self.connect_edge_to_vert(e, v2);

        // Mark edge resources as needing to be recreated
        self.elem_dirty |= super::MeshElementType::EDGES;
        self.index_dirty |= super::MeshElementType::EDGES;
        e
    }

    /// Adds an edge into the disk cycle around a vertex.
    fn connect_edge_to_vert(&mut self, e: id::EdgeId, v: id::VertexId) {
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

    /// Returns the edge between the supplied vertices, or `None`.
    fn query_edge(&self, v_a: id::VertexId, v_b: id::VertexId) -> Option<id::EdgeId> {
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
}

// --- Section: Radial Cycle ---

/// Enables walking over the loops (faces) around an edge
pub struct RadialCycleWalker<'mesh> {
    mesh: &'mesh super::Mesh,
    l_start: id::LoopId,
    l_curr: id::LoopId,
    done: bool,
}

impl<'mesh> RadialCycleWalker<'mesh> {
    fn new(mesh: &'mesh super::Mesh, l_start: id::LoopId) -> Self {
        Self { mesh, l_start, l_curr: l_start, done: false }
    }
}

impl Iterator for RadialCycleWalker<'_> {
    type Item = id::LoopId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let l = self.l_curr;
        self.l_curr = self.mesh[l].radial_next;
        self.done = self.l_curr == self.l_start;
        Some(l)
    }
}

impl DoubleEndedIterator for RadialCycleWalker<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let l = self.l_curr;
        self.l_curr = self.mesh[l].radial_prev;
        self.done = self.l_curr == self.l_start;
        Some(l)
    }
}

impl super::Mesh {
    /// Walks the loops including an edge (faces)
    pub(crate) fn iter_edge_loops(&self, e: id::EdgeId) -> Option<RadialCycleWalker> {
        Some(RadialCycleWalker::new(self, self[e].l?))
    }
}
