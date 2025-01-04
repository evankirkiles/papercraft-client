/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
pub struct SharedBindGroupLayouts {
    pub camera_3d: wgpu::BindGroupLayout,
    pub depth_tex: wgpu::BindGroupLayout,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera_3d: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_3d"),
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
            depth_tex: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("depth_tex"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        count: None,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    },
                ],
            }),
        }
    }
}
