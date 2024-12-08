use gpu::{GPUContext, GPUFrameBuffer, GPUTexture};
use winit::dpi::PhysicalSize;

mod engines;
mod gpu;

pub struct Renderer<'window> {
    ctx: gpu::GPUContext<'window>,
    size: PhysicalSize<u32>,
    presentation_fb: gpu::GPUFrameBuffer,
    engine_3d: engines::EngineDim3,
}

impl<'window> Renderer<'window> {
    // Initializes the GPU surface and creates the different engines needed to
    // render views of the screen.
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits {
                        // TODO: Abstract this into its own config class
                        max_buffer_size: adapter.limits().max_buffer_size,
                        max_texture_dimension_2d: adapter
                            .limits()
                            .max_texture_dimension_2d,
                        ..(if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        })
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        // Surface refers to the capabilities of the surface we're rendering to.
        // On web, this is a WebGL2Context or the equivalent in WebGPU land
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Store the above GPU abstractions into a single context object we
        // can pass around in the future.
        let ctx = GPUContext {
            device,
            config,
            surface,
            queue,
        };
        ctx.configure_surface();

        Self {
            presentation_fb: GPUFrameBuffer::from_swapchain(&ctx),
            engine_3d: engines::EngineDim3::new(&ctx),
            ctx,
            size: PhysicalSize { width, height },
        }
    }

    /// Draws all of the renderables to the screen
    pub fn render(&mut self) -> Result<(), anyhow::Error> {
        self.presentation_fb.render(&self.ctx, |render_pass| {
            // draw from each engine in the presentation render pass.
            self.engine_3d.draw_sync(render_pass);
        })
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = PhysicalSize { width, height };
            self.ctx.resize(width, height);
            self.presentation_fb.resize(&self.ctx);
        }
    }
}
