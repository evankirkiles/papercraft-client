use super::{gpu_batch::GPUBatch, GPUMaterial};

pub struct GPUModel {
    pub materials: Vec<GPUMaterial>,
    pub batches: Vec<GPUBatch>, // Should be same length as above materials vec
}

impl GPUModel {
    pub fn new(device: &wgpu::Device, model: pp_core::model::Model) -> Self {
        let batches = model
            .meshes
            .iter()
            .map(|mesh| GPUBatch::new(device, mesh))
            .collect();

        GPUModel {
            materials: Vec::new(),
            batches,
        }
    }
}
