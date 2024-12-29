use batches::MeshBufferBatches;
use buffers::MeshBuffers;

pub mod batches;
mod buffers;

/// A manager for VBOs / IBOs derived from a mesh.
pub struct MeshGPU {
    /// The GPU resources maintained for this mesh
    pub bufs: MeshBuffers,
    /// Cached "batches" of buffers used for drawing different types of elements
    pub batches: MeshBufferBatches,
}

impl MeshGPU {
    pub fn new(mesh: &pp_core::mesh::Mesh) -> Self {
        let bufs = MeshBuffers::new(mesh);
        let batches = MeshBufferBatches::new(&bufs);
        Self { bufs, batches }
    }
}
