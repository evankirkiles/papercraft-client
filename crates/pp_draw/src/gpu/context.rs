use wgpu::util::DeviceExt;

use super::{layouts::SharedLayouts, settings::Settings};

const BUF_RECT_CONTENTS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

/// A GPU Context owns the resources connected to a Surface's lifetime. It is
/// created when the Renderer is created and used to pass around shared
/// fields for allocating / communicating with the GPU.
#[derive(Debug)]
pub struct Context<'ctx> {
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'ctx>,
    pub queue: wgpu::Queue,
    /// Common wgpu layouts of various types for re-use across programs
    pub shared_layouts: SharedLayouts,
    /// Common buffers (e.g. rect)
    pub buf_rect: wgpu::Buffer,
    /// Global configuration for draw calls
    pub settings: Settings,
}

impl<'ctx> Context<'ctx> {
    pub fn new(
        device: wgpu::Device,
        config: wgpu::SurfaceConfiguration,
        surface: wgpu::Surface<'ctx>,
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
