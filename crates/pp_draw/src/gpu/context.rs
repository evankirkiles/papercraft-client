/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
pub struct SharedBindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera".into()),
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
        });
        Self { camera }
    }
}

/// A GPU Context owns the resources connected to a Surface's lifetime. It is
/// created when the Renderer is created and used to pass around shared
/// fields for allocating / communicating with the GPU.
pub struct Context<'ctx> {
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'ctx>,
    pub queue: wgpu::Queue,
    pub shared_bind_group_layouts: SharedBindGroupLayouts,
}

impl<'ctx> Context<'ctx> {
    pub fn new(
        device: wgpu::Device,
        config: wgpu::SurfaceConfiguration,
        surface: wgpu::Surface<'ctx>,
        queue: wgpu::Queue,
    ) -> Self {
        let shared_bind_group_layouts = SharedBindGroupLayouts::new(&device);
        let ctx = Context { device, config, surface, queue, shared_bind_group_layouts };
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
