use cache::{viewport::BindableViewport, DrawCache};
use pp_core::measures::Dimensions;
use select::{SelectionQueryArea, SelectionQueryResult};
use std::iter;
use wgpu::util::new_instance_with_webgpu_detection;

mod cache;
mod draw;
mod gpu;

pub mod engines;
pub mod select;

#[derive(Debug)]
pub struct Renderer<'window> {
    /// Common context shared for all GPU operations (device, surface)
    pub ctx: gpu::Context<'window>,
    // Textures used as attachments in pipelines
    textures: RendererAttachmentTextures,
    /// The storage manager for all updatable GPU resources (mesh, materials)
    draw_cache: cache::DrawCache,

    /// Manages querying the GPU for pixels containing element indices to select
    select: select::SelectManager,

    /// The core renderer for viewport content (2D and 3D)
    engine_ink: engines::ink::InkEngine,
    /// Renderer for overlays, basically non-mesh things
    engine_overlay: engines::overlay::OverlayEngine,
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
            engine_ink: engines::ink::InkEngine::new(&ctx),
            engine_overlay: engines::overlay::OverlayEngine::new(&ctx),
            ctx,
        }
    }

    /// Synchronizes the DrawCache with the App's current state.
    pub fn prepare(&mut self, state: &mut pp_core::State, editor: &mut pp_editor::Editor) {
        self.draw_cache.prepare_meshes(&self.ctx, state);
        self.draw_cache.prepare_materials(&self.ctx, state);
        self.draw_cache.prepare_print(&self.ctx, state);
        self.draw_cache.prepare_settings(&self.ctx, editor);
        self.draw_cache.prepare_viewports(&self.ctx, editor);
        self.draw_cache.prepare_tool(&self.ctx, editor);
    }

    /// Draws all of the renderables to the screen in each viewport
    pub fn render(&self, state: &pp_core::State) {
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

            // Iterate the active viewports and render their corresponding items
            self.draw_cache.settings.bind(&mut render_pass);
            self.draw_cache.viewports.iter().for_each(|(v_id, viewport)| {
                use cache::viewport::ViewportGPU;
                viewport.bind(&mut render_pass);
                match viewport {
                    ViewportGPU::Folding(_) => self.draw_folding(&state.settings, &mut render_pass),
                    ViewportGPU::Cutting(_) => self.draw_cutting(&state.settings, &mut render_pass),
                }
                self.draw_cache.active_tool.as_ref().filter(|tool| tool.viewport == v_id).inspect(
                    |tool| {
                        self.engine_overlay.draw_tool(&self.ctx, &mut render_pass, &tool.tool);
                    },
                );
            });
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
    pub fn resize(&mut self, dimensions: &Dimensions<u32>) {
        if dimensions.width > 0 && dimensions.height > 0 {
            self.ctx.resize(dimensions);
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
