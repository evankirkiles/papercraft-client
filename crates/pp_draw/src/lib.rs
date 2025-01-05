use std::iter;

use cache::{DrawCache, ViewportGPU};
use winit::dpi::PhysicalSize;

mod cache;
mod engines;
mod gpu;

pub struct Renderer<'window> {
    ctx: gpu::Context<'window>,
    size: PhysicalSize<u32>,
    draw_cache: cache::DrawCache,

    // Rendering engines
    engine_ink2: engines::InkEngine2D,
    engine_ink3: engines::InkEngine3D,

    /// MSAA textures
    color_texture: gpu::Texture,
    depth_texture: gpu::Texture,
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
                        max_texture_dimension_2d: adapter.limits().max_texture_dimension_2d,
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
        let ctx = gpu::Context::new(device, config, surface, queue);
        let (color_texture, depth_texture) = Self::create_presentation_textures(&ctx);

        Self {
            engine_ink2: engines::InkEngine2D::new(&ctx),
            engine_ink3: engines::InkEngine3D::new(&ctx),
            draw_cache: DrawCache::new(&ctx),
            size: PhysicalSize { width, height },
            color_texture,
            depth_texture,
            ctx,
        }
    }

    /// Synchronizes the DrawCache with the App's current state.
    pub fn sync(&mut self, state: &mut pp_core::state::State) {
        self.draw_cache.sync_meshes(&self.ctx, state);
        self.draw_cache.sync_materials(&self.ctx, state);
        self.draw_cache.sync_viewports(&self.ctx, state);
    }

    /// Draws all of the renderables to the screen in each viewport
    pub fn draw(&mut self) {
        let output = self.ctx.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("draw") });

        {
            // RENDER PASS 1: Diffuse / Geometry
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("diffuse"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_texture.view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render 3D if viewport has area
            if self.draw_cache.viewport_3d.bind(&mut render_pass).is_ok() {
                // draw from each engine in the presentation render pass.
                self.draw_cache.meshes.values().for_each(|mesh| {
                    self.engine_ink3.draw_mesh(&mut render_pass, mesh);
                });
                self.engine_ink3.draw_overlays(&mut render_pass);
            }

            // Render 2D if viewport has area
            if self.draw_cache.viewport_2d.bind(&mut render_pass).is_ok() {
                // draw from each engine in the presentation render pass.
                // self.draw_cache.meshes.values().for_each(|mesh| {
                //     self.engine_ink3.draw_mesh(&mut render_pass, mesh);
                // });
                self.engine_ink2.draw_overlays(&mut render_pass);
            }
        }

        self.ctx.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = PhysicalSize { width, height };
            self.ctx.resize(width, height);
            let (color_texture, depth_texture) = Self::create_presentation_textures(&self.ctx);
            self.color_texture = color_texture;
            self.depth_texture = depth_texture;
        }
    }

    fn create_presentation_textures(ctx: &gpu::Context) -> (gpu::Texture, gpu::Texture) {
        let color_texture = gpu::Texture::new(
            ctx,
            wgpu::TextureDescriptor {
                label: Some("color_texture"),
                mip_level_count: 1,
                sample_count: (&ctx.settings.msaa_level).into(),
                dimension: wgpu::TextureDimension::D2,
                format: ctx.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
                size: wgpu::Extent3d {
                    width: ctx.config.width,
                    height: ctx.config.height,
                    depth_or_array_layers: 1,
                },
            },
        );
        let depth_texture = gpu::Texture::new(
            ctx,
            wgpu::TextureDescriptor {
                label: Some("depth_texture"),
                mip_level_count: 1,
                sample_count: (&ctx.settings.msaa_level).into(),
                dimension: wgpu::TextureDimension::D2,
                format: gpu::Texture::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
                size: wgpu::Extent3d {
                    width: ctx.config.width,
                    height: ctx.config.height,
                    depth_or_array_layers: 1,
                },
            },
        );
        (color_texture, depth_texture)
    }
}
