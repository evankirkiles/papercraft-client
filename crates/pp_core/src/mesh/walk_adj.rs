use super::Mesh;
use crate::id::{EdgeId, LoopId, VertexId};

// --- Section: Disk Cycle ---

/// Enables walking over the edges around a vertex
pub struct DiskCycleWalker<'mesh> {
    mesh: &'mesh Mesh,
    v: VertexId,
    e_start: EdgeId,
    e_curr: EdgeId,
    done: bool,
}

impl<'mesh> DiskCycleWalker<'mesh> {
    pub fn new(mesh: &'mesh Mesh, e_start: EdgeId, v: VertexId) -> Self {
        Self { mesh, v, e_start, e_curr: e_start, done: false }
    }
}

impl<'mesh> Iterator for DiskCycleWalker<'mesh> {
    type Item = EdgeId;

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

impl<'mesh> DoubleEndedIterator for DiskCycleWalker<'mesh> {
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

// --- Section: Radial Cycle ---

/// Enables walking over the loops within a face
pub struct RadialCycleWalker<'mesh> {
    mesh: &'mesh Mesh,
    l_start: LoopId,
    l_curr: LoopId,
    done: bool,
}

impl<'mesh> RadialCycleWalker<'mesh> {
    pub fn new(mesh: &'mesh Mesh, l_start: LoopId) -> Self {
        Self { mesh, l_start, l_curr: l_start, done: false }
    }
}

impl<'mesh> Iterator for RadialCycleWalker<'mesh> {
    type Item = LoopId;

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

impl<'mesh> DoubleEndedIterator for RadialCycleWalker<'mesh> {
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

// --- Section: Loop Cycle ---

/// LoopCycle: Enables walking over the loops within a face
pub struct LoopCycleWalker<'mesh> {
    mesh: &'mesh Mesh,
    l_start: LoopId,
    l_curr: LoopId,
    done: bool,
}

impl<'mesh> LoopCycleWalker<'mesh> {
    pub fn new(mesh: &'mesh Mesh, l_start: LoopId) -> Self {
        Self { mesh, l_start, l_curr: l_start, done: false }
    }
}

impl<'mesh> Iterator for LoopCycleWalker<'mesh> {
    type Item = LoopId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let l = self.l_curr;
        self.l_curr = self.mesh[l].next;
        self.done = self.l_curr == self.l_start;
        Some(l)
    }
}

impl<'mesh> DoubleEndedIterator for LoopCycleWalker<'mesh> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let l = self.l_curr;
        self.l_curr = self.mesh[l].prev;
        self.done = self.l_curr == self.l_start;
        Some(l)
    }
}
