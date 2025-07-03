use bitflags::bitflags;
use slotmap::{new_key_type, SlotMap};
use stable_vec::StableVec;
use std::ops;

use crate::id::{EdgeId, FaceId, Id, LoopId, MeshId, PieceId, VertexId};

pub mod edge;
pub mod face;
pub mod loop_;
pub mod matslot;
pub mod piece;
mod primitives;
mod vertex;

use edge::*;
use face::*;
use loop_::*;
use matslot::*;
use piece::*;
use vertex::*;

new_key_type! {
    pub struct MaterialSlotId;
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MeshElementType: u8 {
        const VERTS = 1 << 0;
        const EDGES = 1 << 1;
        const FACES = 1 << 2;
        const LOOPS = 1 << 3;
        const PIECES = 1 << 4;
        const FLAPS = 1 << 5;
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
    pub label: String,

    pub verts: StableVec<Vertex>,
    pub edges: StableVec<Edge>,
    pub faces: StableVec<Face>,
    pub loops: StableVec<Loop>,
    pub pieces: StableVec<Piece>,
    pub material_slots: SlotMap<MaterialSlotId, MaterialSlot>,

    /// Indicates which type of element has changed in this mesh
    pub elem_dirty: MeshElementType,
    pub index_dirty: MeshElementType,
}

impl Mesh {
    pub fn new(label: String) -> Self {
        Self {
            label,
            verts: StableVec::new(),
            edges: StableVec::new(),
            faces: StableVec::new(),
            loops: StableVec::new(),
            pieces: StableVec::new(),
            material_slots: SlotMap::with_key(),
            elem_dirty: MeshElementType::empty(),
            index_dirty: MeshElementType::empty(),
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
impl_index!(PieceId, pieces, Piece);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_duplicate_edge() {
        let mut mesh = Mesh::new(String::from("test"));
        let v1 = mesh.add_vertex([1.0, 0.0, 0.0]);
        let v2 = mesh.add_vertex([0.0, 1.0, 0.0]);
        let e1 = mesh.add_edge(v1, v2);
        let e2 = mesh.add_edge(v2, v1);
        assert_eq!(e1, e2);
        assert_eq!(mesh.verts.num_elements(), 2);
        assert_eq!(mesh.edges.num_elements(), 1);
    }

    #[test]
    fn add_duplicate_face() {
        let mut mesh = Mesh::new(String::from("test"));
        let v1 = mesh.add_vertex([1.0, 0.0, 0.0]);
        let v2 = mesh.add_vertex([0.0, 1.0, 0.0]);
        let v3 = mesh.add_vertex([0.0, 0.0, 1.0]);
        let f1 = mesh.add_face(&[v1, v2, v3], &Default::default());
        let f2 = mesh.add_face(&[v1, v3, v2], &Default::default());
        assert_eq!(f1, f2);
        assert_eq!(mesh.verts.num_elements(), 3);
        assert_eq!(mesh.edges.num_elements(), 3);
        assert_eq!(mesh.loops.num_elements(), 3);
        assert_eq!(mesh.faces.num_elements(), 1);
    }
}
