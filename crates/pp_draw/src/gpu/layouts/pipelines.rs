use super::bind_groups;

/// Shared pipeline layouts to re-use to avoid needing to update bind group
/// descriptions everywhere.
pub struct SharedPipelineLayouts {
    pub pipeline_3d: wgpu::PipelineLayout,
}

impl SharedPipelineLayouts {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &bind_groups::SharedBindGroupLayouts,
    ) -> Self {
        Self {
            pipeline_3d: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_3d"),
                bind_group_layouts: &[&bind_group_layouts.camera_3d],
                push_constant_ranges: &[],
            }),
        }
    }
}