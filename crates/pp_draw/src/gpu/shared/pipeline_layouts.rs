use super::bind_group_layouts::{self, SharedBindGroupLayouts};

/// Shared pipeline layouts to re-use to avoid needing to update bind group
/// descriptions everywhere.
#[derive(Debug)]
pub struct SharedPipelineLayouts {
    pub folding_overlays: wgpu::PipelineLayout,
    pub folding_surface: wgpu::PipelineLayout,
}

impl SharedPipelineLayouts {
    pub fn new(device: &wgpu::Device, bind_group_layouts: &SharedBindGroupLayouts) -> Self {
        Self {
            folding_overlays: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_3d"),
                bind_group_layouts: &[
                    &bind_group_layouts.viewport_folding,
                    &bind_group_layouts.piece,
                ],
                push_constant_ranges: &[],
            }),
            folding_surface: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("surface"),
                bind_group_layouts: &[
                    &bind_group_layouts.viewport_folding,
                    &bind_group_layouts.piece,
                    &bind_group_layouts.material,
                ],
                push_constant_ranges: &[],
            }),
        }
    }
}
