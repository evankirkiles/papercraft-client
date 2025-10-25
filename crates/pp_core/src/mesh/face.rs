use std::collections::HashSet;
use std::collections::VecDeque;

use cgmath::InnerSpace;

use crate::id::{self, Id};
use crate::MaterialId;

use super::loop_::*;
use super::MeshElementType;

/// Input parameters when creating a face
#[derive(Debug, Clone, Copy, Default)]
pub struct FaceDescriptor<'a> {
    pub m: Option<MaterialId>,
    pub nos: Option<&'a [[f32; 3]; 3]>,
    pub uvs: Option<&'a [[f32; 2]; 3]>,
}

/// A face, formed by three vertices and three edges.
#[derive(Debug, Clone, Copy, Default)]
pub struct Face {
    /// Face normal
    pub no: [f32; 3],
    /// Any loop in this face, allowing for loop cycle iteration
    pub l: id::LoopId,
    /// The piece this face is a part of, if any
    pub p: Option<id::PieceId>,
    /// The material of this face, or none if using the default material
    pub m: Option<MaterialId>,
}

impl super::Mesh {
    /// Adds an ngon / face between any number of vertices. If a face already
    /// existed between the verts, returns that face instead.
    pub fn add_face(
        &mut self,
        verts: &[id::VertexId; 3],
        descriptor: &FaceDescriptor,
    ) -> id::FaceId {
        let &FaceDescriptor { uvs, m, nos } = descriptor;
        // If face already exists, return it
        if let Some(f_id) = self.query_face(verts) {
            return f_id;
        }

        // Otherwise, begin creating the face
        let f = id::FaceId::from_usize(self.faces.push(Face { m, ..Default::default() }));

        // Calculate face normal
        let v0 = cgmath::Vector3::from(self[verts[0]].po);
        let v1 = cgmath::Vector3::from(self[verts[1]].po);
        let v2 = cgmath::Vector3::from(self[verts[2]].po);
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        self[f].no = normal.into();

        // Create or use existing edges between all adjacent vertices
        let edges: Vec<id::EdgeId> =
            (0..3).map(|i| self.add_edge(verts[i], verts[(i + 1) % 3])).collect();

        // Create the loops for the face, adding loop + radial data
        let l_start = id::LoopId::from_usize(self.loops.push(Loop {
            f,
            e: edges[0],
            v: verts[0],
            uv: uvs.map(|arr| arr[0]).unwrap_or_default(),
            no: nos.map(|arr| arr[0]).unwrap_or_default(),
            ..Default::default()
        }));
        self.connect_loop_to_edge(l_start, edges[0]);
        self[f].l = l_start;
        let mut l_last = l_start;
        for i in 1..3 {
            let l = id::LoopId::from_usize(self.loops.push(Loop {
                f,
                e: edges[i],
                v: verts[i],
                uv: uvs.map(|arr| arr[i]).unwrap_or_default(),
                no: nos.map(|arr| arr[i]).unwrap_or_default(),
                ..Default::default()
            }));
            self.connect_loop_to_edge(l, edges[i]);
            self[l].prev = l_last;
            self[l_last].next = l;
            l_last = l;
        }
        self[l_start].prev = l_last;
        self[l_last].next = l_start;

        // Face and loop resources need to be recreated
        self.elem_dirty |= MeshElementType::LOOPS | MeshElementType::FACES;
        self.index_dirty |= MeshElementType::LOOPS | MeshElementType::FACES;
        f
    }

    /// Returns the face between the supplied vertices, or None.
    ///
    /// TODO: How do we ensure a consistent vertex order?
    fn query_face(&self, verts: &[id::VertexId]) -> Option<id::FaceId> {
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
                    if l_curr.v == v0 {
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
}

// --- Section: Loop Cycle ---

/// LoopCycle: Enables walking over the loops within a face
pub struct LoopCycleWalker<'mesh> {
    mesh: &'mesh super::Mesh,
    l_start: id::LoopId,
    l_curr: id::LoopId,
    done: bool,
}

impl<'mesh> LoopCycleWalker<'mesh> {
    fn new(mesh: &'mesh super::Mesh, l_start: id::LoopId) -> Self {
        Self { mesh, l_start, l_curr: l_start, done: false }
    }
}

impl Iterator for LoopCycleWalker<'_> {
    type Item = id::LoopId;

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

impl DoubleEndedIterator for LoopCycleWalker<'_> {
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

impl super::Mesh {
    /// Walks the loops in a face (vertices)
    pub fn iter_face_loops(&self, f: id::FaceId) -> LoopCycleWalker {
        LoopCycleWalker::new(self, self[f].l)
    }
}

// --- Section: Connected Face Iterator ---

/// ConnectedFaceWalker: Enables walking over connected faces -- faces not
/// separated by a cut line. Note that cycles may exist here, so if using to
/// find faces for creating a piece, make sure to check its cyclic-ness first.
pub struct ConnectedFaceWalker<'mesh> {
    mesh: &'mesh super::Mesh,
    /// The faces waiting to be explored
    frontier: VecDeque<id::FaceId>,
    /// Faces already explored
    pub visited: HashSet<id::FaceId>,
}

impl<'mesh> ConnectedFaceWalker<'mesh> {
    fn new(mesh: &'mesh super::Mesh, f_start: id::FaceId) -> Self {
        Self { mesh, visited: HashSet::from([f_start]), frontier: VecDeque::from([f_start]) }
    }
}

impl Iterator for ConnectedFaceWalker<'_> {
    type Item = id::FaceId;

    fn next(&mut self) -> Option<Self::Item> {
        let f_id = self.frontier.pop_front()?;
        // Expand the frontier to include unvisited faces adjacent to this face
        self.frontier.extend(
            self.mesh
                .iter_face_loops(f_id)
                .filter_map(|l| {
                    // Do not traverse across cut edges
                    let e_id = self.mesh[l].e;
                    if self.mesh[e_id].cut.is_none() {
                        self.mesh.iter_edge_loops(e_id)
                    } else {
                        None
                    }
                })
                .flatten()
                .filter_map(|l_id| {
                    // Only visit unvisited faces (faces not already in visited)
                    let f_id = self.mesh[l_id].f;
                    self.visited.insert(f_id).then_some(f_id)
                }),
        );
        Some(f_id)
    }
}

impl super::Mesh {
    /// Walks all faces connected to the given face, respecting cut boundaries.
    pub fn iter_connected_faces(&self, f: id::FaceId) -> ConnectedFaceWalker {
        ConnectedFaceWalker::new(self, f)
    }
}
