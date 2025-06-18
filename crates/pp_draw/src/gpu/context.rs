use wgpu::util::DeviceExt;

use super::{layouts::SharedLayouts, settings::Settings};

// Simple triangle strip quad
const BUF_RECT_CONTENTS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];
// Simple outline of a rect, with each segment itself made of a quad. We could
// compress this down to 14 verts by using miters and a triangle strip, but... whatever
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

/// A GPU Context owns the resources connected to a Surface's lifetime. It is
/// created when the Renderer is created and used to pass around shared
/// fields for allocating / communicating with the GPU.
#[derive(Debug)]
pub(crate) struct Context<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub view_format: wgpu::TextureFormat,
    /// Common wgpu layouts of various types for re-use across programs
    pub shared_layouts: SharedLayouts,
    /// Common buffers (e.g. rect)
    pub buf_rect: wgpu::Buffer,
    pub buf_rect_outline: wgpu::Buffer,
    /// Global configuration for draw calls
    pub settings: Settings,
}

impl<'window> Context<'window> {
    pub fn new(
        device: wgpu::Device,
        config: wgpu::SurfaceConfiguration,
        surface: wgpu::Surface<'window>,
        queue: wgpu::Queue,
    ) -> Self {
        let ctx = Context {
            shared_layouts: SharedLayouts::new(&device),
            settings: Settings::default(),
            buf_rect: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ctx.buf_rect"),
                contents: bytemuck::bytes_of(&BUF_RECT_CONTENTS),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            buf_rect_outline: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ctx.buf_rect"),
                contents: bytemuck::bytes_of(&BUF_RECT_OUTLINE_CONTENTS),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            view_format: config.view_formats[0],
            device,
            config,
            surface,
            queue,
        };
        ctx.configure_surface();
        ctx
    }

    pub fn configure_surface(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    /// Reconfigures the surface for the specified width and heights
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.configure_surface();
    }
}
