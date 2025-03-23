use bitflags::bitflags;
use stable_vec::StableVec;
use std::ops;

use crate::id::{EdgeId, FaceId, Id, LoopId, MeshId, VertexId};

mod edge;
mod face;
mod loop_;
mod primitives;
mod vertex;

use edge::*;
use face::*;
use loop_::*;
use vertex::*;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MeshElementType: u8 {
        const VERTS = 1 << 0;
        const EDGES = 1 << 1;
        const FACES = 1 << 2;
        const LOOPS = 1 << 3;
    }
}

impl From<MeshElementType> for bool {
    fn from(value: MeshElementType) -> Self {
        !value.is_empty()
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
#[derive(Debug)]
pub struct Mesh {
    pub id: MeshId,
    pub label: String,

    pub verts: StableVec<Vertex>,
    pub edges: StableVec<Edge>,
    pub faces: StableVec<Face>,
    pub loops: StableVec<Loop>,

    /// Indicates which type of element has changed in this mesh
    pub elem_dirty: MeshElementType,
    pub index_dirty: MeshElementType,
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
            elem_dirty: MeshElementType::empty(),
            index_dirty: MeshElementType::empty(),
        }
    }

    // --- Section: Lazy Calculations ---

    /// Makes sure that element indices are up-to-date to prepare for IBO
    /// creation.
    pub fn ensure_elem_index(&mut self, el_types: MeshElementType) {
        let el_types_and_dirty = el_types & self.index_dirty;
        if (MeshElementType::VERTS & el_types_and_dirty).into() {
            self.verts.values_mut().enumerate().for_each(|(i, el)| {
                el.index = Some(i);
            });
        }
        if (MeshElementType::EDGES & el_types_and_dirty).into() {
            self.edges.values_mut().enumerate().for_each(|(i, el)| {
                el.index = Some(i);
            });
        }
        if ((MeshElementType::FACES | MeshElementType::LOOPS) & el_types_and_dirty).into() {
            self.faces.values_mut().enumerate().for_each(|(i, el)| {
                el.index = Some(i);
            });
        }
        if (MeshElementType::LOOPS & el_types_and_dirty).into() {
            let loops: Vec<_> = self
                .faces
                .indices()
                .flat_map(|f| self.iter_face_loops(FaceId::from_usize(f)))
                .collect();
            loops.iter().enumerate().for_each(|(i, l)| {
                self[*l].index = Some(i);
            });
        }
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
