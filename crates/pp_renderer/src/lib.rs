use viewport_3d::Viewport3d;
use winit::dpi::PhysicalSize;

mod camera;
mod texture;
pub mod viewport_3d;

pub trait Renderable {
    /// Draws the region to the surface within render_pass
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}

pub struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    depth_texture: texture::Texture,
}

impl<'window> Renderer<'window> {
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
        surface.configure(&device, &config);

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            &config,
            "Depth Texture",
        );

        Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            size: PhysicalSize { width, height },
        }
    }

    /// Draws all of the renderables to the screen
    pub fn render(
        &mut self,
        viewports: &[viewport_3d::Viewport3d],
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Renderer Encoder"),
            },
        );
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    // depth_stencil_attachment: Some(
                    //     wgpu::RenderPassDepthStencilAttachment {
                    //         view: &self.depth_texture.view,
                    //         depth_ops: Some(wgpu::Operations {
                    //             load: wgpu::LoadOp::Clear(1.0),
                    //             store: wgpu::StoreOp::Store,
                    //         }),
                    //         stencil_ops: None,
                    //     },
                    // ),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            log::warn!("Rendering");

            // run each renderable on the render pass.
            for viewport in viewports {
                viewport.render(&mut render_pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = PhysicalSize { width, height };
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            );
        }
    }

    pub fn add_viewport(
        &self,
        dims: viewport_3d::Rect,
    ) -> viewport_3d::Viewport3d {
        Viewport3d::new(&self.device, &self.config, dims)
    }
}
