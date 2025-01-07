use std::iter;

pub use crate::engines::select::SelectionMask;
use crate::{
    cache::{self, ViewportGPU},
    engines::select::SelectEngine,
    gpu,
};

/// An area on which to perform a selection action
#[derive(Debug, Copy, Clone)]
pub struct SelectionRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A single query submitted to the GPU to populate the
#[derive(Debug, Copy, Clone)]
pub struct SelectionQuery {
    pub mask: SelectionMask,
    pub rect: SelectionRect,
}

/// Represents whether a selection query is currently being processed.
#[derive(Debug, Clone)]
pub enum SelectManagerQueryState {
    Idle,
    InFlight { index: wgpu::SubmissionIndex },
    Ready(SelectionQuery),
}

pub struct SelectManager {
    textures: SelectManagerAttachmentTextures,

    // Rendering engines
    select_engine: SelectEngine,

    // Reading back of selection state
    pub query_state: SelectManagerQueryState,
    pub select_buf: wgpu::Buffer,
}

#[derive(Debug)]
pub enum SelectionQueryError {
    QueryInFlight,
}

impl SelectManager {
    pub fn new(ctx: &gpu::Context) -> Self {
        let textures = SelectManagerAttachmentTextures::create(ctx);
        let select_buf = textures.get_buf(ctx);
        Self {
            select_engine: SelectEngine::new(ctx),
            select_buf,
            textures,
            query_state: SelectManagerQueryState::Idle,
        }
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, ctx: &gpu::Context) {
        // Block until any in-flight selection queries have been processed, as
        // they expect a corresponding buffer to map into (which we are recreating)
        match self.query_state {
            SelectManagerQueryState::Idle => {}
            SelectManagerQueryState::InFlight { .. } => {
                while !ctx.device.poll(wgpu::MaintainBase::Wait).is_queue_empty() {}
            }
            SelectManagerQueryState::Ready(_) => {
                self.select_buf.unmap();
            }
        }
        self.query_state = SelectManagerQueryState::Idle;
        self.textures = SelectManagerAttachmentTextures::create(ctx);
        self.select_buf = self.textures.get_buf(ctx);
    }

    /// Queries for `ID`s of element drawn within the supplied rect
    ///
    /// Note that this depends on a asynchronous process of draw calls, texture to
    /// buffer copying, and then buffer mapping to get the data from the GPU to
    /// the CPU. For this reason, you must pass your operation as a command
    /// alongside an event loop - once the GPU marks the buffer as mapped and
    /// ready to be used, an event will be emitted onto the event loop for
    /// further processing.
    pub fn submit_query(
        &mut self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        query: SelectionQuery,
        callback: impl Fn(SelectManagerQueryState) + wgpu::WasmNotSend + 'static,
    ) -> Result<wgpu::SubmissionIndex, SelectionQueryError> {
        match self.query_state {
            SelectManagerQueryState::Idle => {}
            SelectManagerQueryState::Ready(_) => self.select_buf.unmap(),
            SelectManagerQueryState::InFlight { .. } => {
                return Err(SelectionQueryError::QueryInFlight);
            }
        };
        let SelectionQuery { rect, mask } = query;

        // Begin render pass to draw into the selection buffer
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("select") });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("select"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.textures.idx.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
            // Set the scissor for the active viewport so we don't rasterize
            // any unnecessary pixels, just the ones which we'll check for selection.
            render_pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);

            // Render 3D if viewport has area
            if draw_cache.viewport_3d.bind(&mut render_pass).is_ok() {
                // draw from each engine in the presentation render pass.
                draw_cache.meshes.values().for_each(|mesh| {
                    self.select_engine.draw_mesh(&mut render_pass, mesh, mask);
                });
            }

            // Render 2D if viewport has area
            if draw_cache.viewport_2d.bind(&mut render_pass).is_ok() {
                // draw from each engine in the presentation render pass.
                // self.draw_cache.meshes.values().for_each(|mesh| {
                //     self.engine_ink3.draw_mesh(&mut render_pass, mesh);
                // });
            }
        }

        // After render pass completes, copy the desired region of the texture
        // into the select buf, all still on the GPU
        let block_size = TEX_FORMAT.block_copy_size(None).unwrap();
        let bytes_per_row = self.textures.idx.texture.width() * block_size;
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTextureBase {
                aspect: wgpu::TextureAspect::All,
                texture: &self.textures.idx.texture,
                origin: wgpu::Origin3d { x: rect.x, y: rect.y, z: 0 },
                mip_level: 0,
            },
            wgpu::ImageCopyBufferBase {
                buffer: &self.select_buf,
                layout: wgpu::ImageDataLayout {
                    offset: (rect.y * bytes_per_row + rect.x * block_size).into(),
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.textures.idx.texture.height()),
                },
            },
            wgpu::Extent3d { width: rect.width, height: rect.height, depth_or_array_layers: 1 },
        );

        // Submit all commands and track the SubmissionIndex for completion
        let submission_index = ctx.queue.submit(iter::once(encoder.finish()));
        self.query_state = SelectManagerQueryState::InFlight { index: submission_index.clone() };

        // Once the buffer has been written to on the GPU, map the relevant
        // portion back into CPU-land and run the callback using it.
        self.select_buf.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            result.expect("map_async failed");
            callback(SelectManagerQueryState::Ready(query));
        });
        ctx.device.poll(wgpu::MaintainBase::Poll);

        Ok(submission_index)
    }

    /// Must be called once the select buffer is successfully mapped to allow
    /// the program to be resized again / perform another selection query.
    pub fn recv_query(&mut self, ctx: &gpu::Context, query: SelectionQuery) {
        if let SelectManagerQueryState::InFlight { index } = &self.query_state {
            ctx.device.poll(wgpu::MaintainBase::WaitForSubmissionIndex(index.clone()));
            self.query_state = SelectManagerQueryState::Ready(query);
        }
    }
}

struct SelectManagerAttachmentTextures {
    // Object picking / select textures
    idx: gpu::Texture,
    depth: gpu::Texture,
}

/// The format of the selection idx texture.
pub const TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

/// Rounds a number up to the nearest multiple of `align`
pub const fn align_up(num: u32, align: u32) -> u32 {
    ((num) + ((align) - 1)) & !((align) - 1)
}

impl SelectManagerAttachmentTextures {
    fn create(ctx: &gpu::Context) -> Self {
        // Align the width of the image up to a 256-byte alignment per row, as
        // required to use `copy_texture_to_buffer`. This will not affect the
        // final image, as we always set the viewport before rendering.
        let block_size = TEX_FORMAT.block_copy_size(None).unwrap();
        let size = wgpu::Extent3d {
            width: align_up(ctx.config.width * block_size, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                / block_size,
            height: ctx.config.height,
            depth_or_array_layers: 1,
        };
        Self {
            idx: gpu::Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("select.idx"),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Uint,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                    size,
                },
            ),
            depth: gpu::Texture::new(
                ctx,
                wgpu::TextureDescriptor {
                    label: Some("select.depth"),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: gpu::Texture::DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                    size,
                },
            ),
        }
    }

    /// Returns a corresponding buf to be copied into from the texture
    fn get_buf(&self, ctx: &gpu::Context) -> wgpu::Buffer {
        let block_size = TEX_FORMAT.block_copy_size(None).unwrap();
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("select.idx_buf"),
            size: (self.depth.texture.width() * self.depth.texture.height() * block_size).into(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }
}
