use cache::DrawCache;
use pp_core::tool::PhysicalDimensions;
use select::{SelectionQueryArea, SelectionQueryResult};
use std::iter;
use wgpu::util::new_instance_with_webgpu_detection;

mod cache;
mod gpu;

pub mod engines;
pub mod select;

#[derive(Debug)]
pub struct Renderer<'window> {
    /// Common context shared for all GPU operations (device, surface)
    ctx: gpu::Context<'window>,
    // Textures used as attachments in pipelines
    textures: RendererAttachmentTextures,
    /// Manages querying the GPU for pixels containing element indices to select
    select: select::SelectManager,
    /// A storage manager for all updatable GPU resources (mesh, materials)
    draw_cache: cache::DrawCache,
    /// The core renderer for viewport content (2D and 3D)
    draw_engine: engines::ink::InkEngine,
}

impl<'window> Renderer<'window> {
    // Initializes the GPU surface and creates the different engines needed to
    // render views of the screen.
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            ..Default::default()
        })
        .await;

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        adapter.get_downlevel_capabilities();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    // TODO: Abstract this into its own config class
                    max_buffer_size: adapter.limits().max_buffer_size,
                    max_texture_dimension_2d: adapter.limits().max_texture_dimension_2d,
                    ..wgpu::Limits::downlevel_webgl2_defaults()
                },
                label: None,
                trace: wgpu::Trace::Off,
                memory_hints: Default::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let mut format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let mut view_format = format;
        let mut clear_color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

        // If running on WebGPU, use SRGB textures even for non-SRGB surface,
        // and use a different "clear" color as WebGPU canvases seems to be opaque.
        // In the future, we should make SRGB an optional thing.
        if adapter.get_downlevel_capabilities().is_webgpu_compliant() {
            format = format.remove_srgb_suffix();
            view_format = format.add_srgb_suffix();
            clear_color = wgpu::Color { r: 0.005, g: 0.005, b: 0.005, a: 0.0 };
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![view_format],
            desired_maximum_frame_latency: 2,
        };

        // Store the above GPU abstractions into a single context object we
        // can pass around in the future.
        let ctx = gpu::Context::new(device, config, surface, queue, clear_color);

        Self {
            select: select::SelectManager::new(&ctx),
            textures: RendererAttachmentTextures::create(&ctx),
            draw_cache: DrawCache::new(&ctx),
            draw_engine: engines::ink::InkEngine::new(&ctx),
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
    pub fn draw(&self, state: &pp_core::State) {
        let output = self.ctx.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: (self.ctx.view_format != self.ctx.config.format)
                .then_some(self.ctx.view_format),
            ..Default::default()
        });
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("draw") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("draw"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: if self.ctx.settings.msaa_level != gpu::settings::MSAALevel::None {
                        &self.textures.color.view
                    } else {
                        &view
                    },
                    resolve_target: (self.ctx.settings.msaa_level
                        != gpu::settings::MSAALevel::None)
                        .then_some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.ctx.clear_color),
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
            if self.draw_cache.viewport_3d.bind(&mut render_pass) {
                self.draw_cache.common.piece_identity.bind(&mut render_pass);
                // First, draw the actual textured surface
                self.draw_cache.materials.iter().for_each(|(id, mat)| {
                    mat.bind(&mut render_pass);
                    self.draw_cache.meshes.values().for_each(|mesh| {
                        self.draw_engine.draw_mesh_for_material(
                            &self.ctx,
                            &mut render_pass,
                            mesh,
                            id,
                        );
                    });
                });
                // Now draw the overlays / not the surface
                self.draw_cache.meshes.values().for_each(|mesh| {
                    self.draw_engine.draw_mesh(
                        &self.ctx,
                        &state.settings,
                        &mut render_pass,
                        mesh,
                        self.draw_cache.viewport_3d.xray_mode,
                    );
                });
                // 2D drawing
                // self.draw_cache.materials.iter().for_each(|(id, mat)| {
                //     mat.bind(&mut render_pass);
                //     self.draw_cache.meshes.values().for_each(|mesh| {
                //         self.draw_engine.draw_piece_mesh_for_material(
                //             &self.ctx,
                //             &mut render_pass,
                //             mesh,
                //             id,
                //         );
                //     });
                // });
                // // Draw the overlays / not the surface
                // self.draw_cache.meshes.values().for_each(|mesh| {
                //     self.draw_engine.draw_piece_mesh(
                //         &self.ctx,
                //         &state.settings,
                //         &mut render_pass,
                //         mesh,
                //     );
                // });
                self.draw_engine.draw_3d_overlays(&self.ctx, &mut render_pass);
            }

            // Render 2D pieces if viewport has area
            if self.draw_cache.viewport_2d.bind(&mut render_pass) {
                self.draw_cache.materials.iter().for_each(|(id, mat)| {
                    mat.bind(&mut render_pass);
                    self.draw_cache.meshes.values().for_each(|mesh| {
                        self.draw_engine.draw_piece_mesh_for_material(
                            &self.ctx,
                            &mut render_pass,
                            mesh,
                            id,
                        );
                    });
                });
                // Draw the overlays / not the surface
                self.draw_cache.meshes.values().for_each(|mesh| {
                    self.draw_engine.draw_piece_mesh(
                        &self.ctx,
                        &state.settings,
                        &mut render_pass,
                        mesh,
                    );
                });
                self.draw_engine.draw_2d_overlays(&self.ctx, &mut render_pass);
            }
        }

        self.ctx.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    /// Queries the selection manager to prepare the supplied rect.
    pub fn select_query<F: Fn(&SelectionQueryArea, &SelectionQueryResult) + 'static>(
        &mut self,
        area: select::SelectionQueryArea,
        callback: Box<F>,
    ) -> Result<(), select::SelectionQueryError> {
        self.select.query(&self.ctx, &self.draw_cache, area, callback)
    }

    /// Polls the select engine to see if there are any fulfilled selection queries.
    /// If there are, this will register their completion and perform any
    /// selection actions they were queried with.
    pub fn select_poll(&mut self) {
        self.select.poll(&self.ctx);
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.ctx.resize(width, height);
            self.select.resize(&self.ctx);
            self.textures = RendererAttachmentTextures::create(&self.ctx);
        }
    }

    /// Gets the current size of the canvas, in pixels
    pub fn curr_size(&self) -> PhysicalDimensions<f32> {
        PhysicalDimensions {
            width: self.ctx.config.width as f32,
            height: self.ctx.config.height as f32,
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
                    format: ctx.view_format,
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
