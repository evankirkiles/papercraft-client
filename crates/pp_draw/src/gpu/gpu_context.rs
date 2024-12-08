/// GPUContext owns the resources connected to a Surface's lifetime. It is
/// created when the Renderer is created and used to pass around shared
/// fields for allocating / communicating with the GPU.
pub struct GPUContext<'ctx> {
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'ctx>,
    pub queue: wgpu::Queue,
}

impl<'ctx> GPUContext<'ctx> {
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
