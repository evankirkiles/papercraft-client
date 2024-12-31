/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
pub struct SharedBindGroupLayouts {
    pub camera_3d: wgpu::BindGroupLayout,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera_3d: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_3d"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
