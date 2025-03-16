use std::iter;

use cache::{DrawCache, ViewportGPU};

mod cache;
mod engines;
mod gpu;

pub mod select;

#[derive(Debug)]
pub struct Renderer<'window> {
    ctx: gpu::Context<'window>,
    draw_cache: cache::DrawCache,

    // Textures used as attachments in pipelines
    textures: RendererAttachmentTextures,

    // Rendering engines
    engine_ink2: engines::ink2::InkEngine2D,
    engine_ink3: engines::ink3::InkEngine3D,

    /// Manages querying the GPU for pixels containing element indices to select
    pub select: select::SelectManager,
}

impl<'window> Renderer<'window> {
    // Initializes the GPU surface and creates the different engines needed to
    // render views of the screen.
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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
                        ..wgpu::Limits::downlevel_webgl2_defaults()
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

        Self {
            engine_ink2: engines::ink2::InkEngine2D::new(&ctx),
            engine_ink3: engines::ink3::InkEngine3D::new(&ctx),
            textures: RendererAttachmentTextures::create(&ctx),
            draw_cache: DrawCache::new(&ctx),
            select: select::SelectManager::new(&ctx),
            ctx,
        }
    }

    /// Synchronizes the DrawCache with the App's current state.
    pub fn sync(&mut self, state: &mut pp_core::State) {
        self.draw_cache.sync_meshes(&self.ctx, state);
        self.draw_cache.sync_materials(&self.ctx, state);
        self.draw_cache.sync_viewports(&self.ctx, state);
    }

    /// Draws all of the renderables to the screen in each viewport
    pub fn draw(&self) {
        let output = self.ctx.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("draw") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("draw"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures.color.view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.000,
                            g: 0.000,
                            b: 0.000,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.textures.depth.view,
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
                    self.engine_ink3.draw_mesh(&self.ctx, &mut render_pass, mesh);
                });
                self.engine_ink3.draw_overlays(&self.ctx, &mut render_pass);
            }

            // Render 2D if viewport has area
            if self.draw_cache.viewport_2d.bind(&mut render_pass).is_ok() {
                // draw from each engine in the presentation render pass.
                // self.draw_cache.meshes.values().for_each(|mesh| {
                //     self.engine_ink3.draw_mesh(&mut render_pass, mesh);
                // });
                self.engine_ink2.draw_overlays(&self.ctx, &mut render_pass);
            }
        }

        self.ctx.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    /// Queries the selection manager to prepare the supplied rect.
    pub fn select_query_submit(
        &mut self,
        query: select::SelectionQuery,
    ) -> Result<(), select::SelectionQueryError> {
        self.select.query_submit(&self.ctx, &self.draw_cache, query)
    }

    /// Polls the select engine to see if there are any fulfilled selection queries.
    /// If there are, this will register their completion and perform any
    /// selection actions they were queried with.
    pub fn select_query_sync(&mut self, state: &mut pp_core::State) {
        self.select.query_sync(&self.ctx, state);
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.ctx.resize(width, height);
            self.select.resize(&self.ctx);
            self.textures = RendererAttachmentTextures::create(&self.ctx);
        }
    }
}

#[derive(Debug)]
struct RendererAttachmentTextures {
    // Display textures (maybe MSAA)
    color: gpu::Texture,
    depth: gpu::Texture,
}

impl RendererAttachmentTextures {
    fn create(ctx: &gpu::Context) -> Self {
        Self {
            color: gpu::Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("renderer.color"),
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
            ),
            depth: gpu::Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("renderer.depth"),
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
            ),
        }
    }
}
