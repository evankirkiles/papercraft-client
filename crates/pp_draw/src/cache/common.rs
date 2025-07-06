use crate::gpu;

use super::mesh::piece::PieceGPU;

#[derive(Debug)]
pub struct CommonGPUResources {
    /// A 4x4 identity matrix to use as the "base" piece transform. Allows us
    /// to use the same pipelines across piece views and non-piece views
    pub piece_identity: PieceGPU,
}

impl CommonGPUResources {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self { piece_identity: PieceGPU::identity(ctx) }
    }
}
