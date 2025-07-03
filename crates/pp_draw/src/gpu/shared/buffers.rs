use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct SharedBuffers {
    pub rect: wgpu::Buffer,
    pub rect_outline: wgpu::Buffer,
}

impl SharedBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            rect: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ctx.buf_rect"),
                contents: bytemuck::bytes_of(&BUF_RECT_CONTENTS),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            rect_outline: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ctx.buf_rect"),
                contents: bytemuck::bytes_of(&BUF_RECT_OUTLINE_CONTENTS),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        }
    }
}

// Triangle strip quad
const BUF_RECT_CONTENTS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

// Outline of a rect, with each segment itself made of a quad. We could compress
// this down to 14 verts by using miters and a triangle strip, but... whatever
const BUF_RECT_OUTLINE_CONTENTS: [[f32; 2]; 24] = [
    // segment 1
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [1.0, 1.0],
    // segment 2
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [1.0, 1.0],
    // segment 3
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [1.0, 1.0],
    // segment 4
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [0.0, 1.0],
    [1.0, 0.0],
    [1.0, 1.0],
];
