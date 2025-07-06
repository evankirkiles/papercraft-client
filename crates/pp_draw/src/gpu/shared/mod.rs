use bind_group_layouts::SharedBindGroupLayouts;
use buffers::SharedBuffers;
use pipeline_layouts::SharedPipelineLayouts;

pub mod bind_group_layouts;
pub mod buffers;
pub mod pipeline_layouts;

#[derive(Debug)]
pub struct SharedGPUResources {
    pub bind_group_layouts: SharedBindGroupLayouts,
    pub pipeline_layouts: SharedPipelineLayouts,
    pub buffers: SharedBuffers,
}

impl SharedGPUResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layouts = SharedBindGroupLayouts::new(device);
        let pipeline_layouts = SharedPipelineLayouts::new(device, &bind_group_layouts);
        Self { bind_group_layouts, pipeline_layouts, buffers: SharedBuffers::new(device) }
    }
}
