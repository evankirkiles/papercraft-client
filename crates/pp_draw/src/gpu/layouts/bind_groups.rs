/// Global ordering of bind groups, so shaders can refer to consistent bind
/// groups without conflict.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum UniformBindGroup {
    Camera,
    Piece,
}

impl UniformBindGroup {
    pub fn value(&self) -> u32 {
        match self {
            UniformBindGroup::Camera => 0,
            UniformBindGroup::Piece => 1,
        }
    }
}

/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
#[derive(Debug)]
pub struct SharedBindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub piece: wgpu::BindGroupLayout,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
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
        }
    }
}
