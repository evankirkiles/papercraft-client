use pp_editor::measures::Dimensions;
use wgpu::util::DeviceExt;

use super::{settings::Settings, shared::SharedGPUResources};

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
pub struct Context<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub view_format: wgpu::TextureFormat,
    pub clear_color: wgpu::Color,
    /// Common wgpu resources of various types for re-use across programs
    pub shared: SharedGPUResources,
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
        clear_color: wgpu::Color,
    ) -> Self {
        let ctx = Context {
            shared: SharedGPUResources::new(&device, &queue),
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
            clear_color,
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
    pub fn resize(&mut self, dimensions: &Dimensions<u32>) {
        self.config.width = dimensions.width;
        self.config.height = dimensions.height;
        self.configure_surface();
    }
}
