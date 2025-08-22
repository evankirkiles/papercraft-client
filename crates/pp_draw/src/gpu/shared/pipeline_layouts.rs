use super::bind_group_layouts::SharedBindGroupLayouts;

/// Shared pipeline layouts to re-use to avoid needing to update bind group
/// descriptions everywhere.
#[derive(Debug)]
pub struct SharedPipelineLayouts {
    pub mesh_surface: wgpu::PipelineLayout,
    pub mesh_overlays: wgpu::PipelineLayout,
}

impl SharedPipelineLayouts {
    pub fn new(device: &wgpu::Device, bind_group_layouts: &SharedBindGroupLayouts) -> Self {
        Self {
            mesh_surface: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mesh_surface"),
                bind_group_layouts: &[
                    &bind_group_layouts.settings,
                    &bind_group_layouts.viewport,
                    &bind_group_layouts.piece,
                    &bind_group_layouts.material,
                ],
                push_constant_ranges: &[],
            }),
            mesh_overlays: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mesh_overlays"),
                bind_group_layouts: &[
                    &bind_group_layouts.settings,
                    &bind_group_layouts.viewport,
                    &bind_group_layouts.piece,
                ],
                push_constant_ranges: &[],
            }),
        }
    }
}
