use crate::cache::viewport::{cutting::CuttingViewportGPU, folding::FoldingViewportGPU};

/// Global ordering of bind groups, so shaders can refer to consistent bind
/// groups without conflict.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BindGroup {
    Viewport,
    Piece,
    Material,
}

impl BindGroup {
    pub fn value(&self) -> u32 {
        match self {
            BindGroup::Viewport => 0,
            BindGroup::Piece => 1,
            BindGroup::Material => 2,
        }
    }
}

/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
#[derive(Debug)]
pub struct SharedBindGroupLayouts {
    pub viewport_cutting: wgpu::BindGroupLayout,
    pub viewport_folding: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub piece: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            viewport_cutting: CuttingViewportGPU::create_bind_group_layout(device),
            viewport_folding: FoldingViewportGPU::create_bind_group_layout(device),
            camera: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
            piece: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("piece"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
            material: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }),
        }
    }
}
