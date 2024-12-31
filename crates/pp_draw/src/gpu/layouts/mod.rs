mod bind_groups;
mod pipelines;

pub struct SharedLayouts {
    pub bind_groups: bind_groups::SharedBindGroupLayouts,
    pub pipelines: pipelines::SharedPipelineLayouts,
}

impl SharedLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_groups = bind_groups::SharedBindGroupLayouts::new(device);
        let pipelines = pipelines::SharedPipelineLayouts::new(device, &bind_groups);
        Self { bind_groups, pipelines }
    }
}
