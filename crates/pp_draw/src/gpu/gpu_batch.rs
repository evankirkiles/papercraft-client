use wgpu::util::DeviceExt;

pub struct GPUBatch {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub len: u32,
}

impl GPUBatch {
    pub fn new(device: &wgpu::Device, mesh: &pp_core::model::Mesh) -> Self {
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.faces),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            len: mesh.faces.len() as u32,
        }
    }
}
