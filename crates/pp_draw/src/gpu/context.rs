use pp_core::measures::Dimensions;

use super::{settings::Settings, shared::SharedGPUResources};

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
            shared: SharedGPUResources::new(&device),
            settings: Settings::default(),
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
