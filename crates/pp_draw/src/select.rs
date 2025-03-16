use std::{cell::RefCell, iter, ops::Deref, rc::Rc};

use pp_core::id::{Id, MeshId, VertexId};

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

/// Once the selection query is ready, what should we do to it?
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum SelectImmediateAction {
    /// Select the valid pixel for selection nearest to the center
    Nearest,
    /// Toggle the selection state of the pixel nearest to the center
    NearestToggle,
    /// Select everything in the queried region
    All,
}

/// A single query submitted to the GPU to populate the
#[derive(Debug, Copy, Clone)]
pub struct SelectionQuery {
    pub action: Option<SelectImmediateAction>,
    pub mask: SelectionMask,
    pub rect: SelectionRect,
}

impl SelectionQuery {
    pub fn contains(&self, other: &SelectionQuery) -> bool {
        self.mask.contains(other.mask)
            && other.rect.x >= self.rect.x
            && other.rect.y >= self.rect.y
            && other.rect.x + other.rect.width <= self.rect.x + self.rect.width
            && other.rect.y + other.rect.height <= self.rect.y + self.rect.height
    }
}

/// Represents whether a selection query is currently being processed.
#[derive(Debug, Clone)]
pub enum SelectManagerQueryState {
    Unmapped,
    InFlight { index: wgpu::SubmissionIndex, query: SelectionQuery },
    UnhandledMapped { index: wgpu::SubmissionIndex, query: SelectionQuery },
    Mapped(SelectionQuery),
}

#[derive(Debug)]
pub struct SelectManager {
    textures: SelectManagerAttachmentTextures,

    // Rendering engines
    select_engine: SelectEngine,

    // Reading back of selection state
    pub query_state: Rc<RefCell<SelectManagerQueryState>>,
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
            query_state: Rc::new(RefCell::new(SelectManagerQueryState::Unmapped)),
        }
    }

    /// Updates the GPUContext for new dimensions
    pub fn resize(&mut self, ctx: &gpu::Context) {
        // Block until any in-flight selection queries have been processed, as
        // they expect a corresponding buffer to map into (which we are recreating)
        match self.query_state.borrow().deref() {
            SelectManagerQueryState::Unmapped => {}
            SelectManagerQueryState::InFlight { .. } => {
                while !ctx.device.poll(wgpu::MaintainBase::Wait).is_queue_empty() {}
            }
            SelectManagerQueryState::Mapped(_)
            | SelectManagerQueryState::UnhandledMapped { .. } => {
                self.select_buf.unmap();
            }
        }
        self.query_state.replace(SelectManagerQueryState::Unmapped);
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
    pub fn query_submit(
        &mut self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        query: SelectionQuery,
    ) -> Result<(), SelectionQueryError> {
        match self.query_state.borrow().deref() {
            SelectManagerQueryState::Unmapped => {}
            SelectManagerQueryState::InFlight { .. } => {
                return Err(SelectionQueryError::QueryInFlight);
            }
            SelectManagerQueryState::Mapped(_)
            | SelectManagerQueryState::UnhandledMapped { .. } => {
                self.select_buf.unmap();
            }
        };
        let SelectionQuery { action: _, rect, mask } = query;

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
                draw_cache.meshes.values().for_each(|mesh| {
                    self.select_engine.draw_mesh(ctx, &mut render_pass, mesh, mask);
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
        let block_size = self.textures.block_size;
        let bytes_per_row = self.textures.idx.texture.width() * block_size;
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.textures.idx.texture,
                origin: wgpu::Origin3d { x: rect.x, y: rect.y, z: 0 },
                mip_level: 0,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.select_buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: (rect.y * bytes_per_row + rect.x * block_size).into(),
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.textures.idx.texture.height()),
                },
            },
            wgpu::Extent3d { width: rect.width, height: rect.height, depth_or_array_layers: 1 },
        );

        // Submit all commands and track the SubmissionIndex for completion
        let index = ctx.queue.submit(iter::once(encoder.finish()));
        self.query_state.replace(SelectManagerQueryState::InFlight { index: index.clone(), query });

        // Once the buffer has been written to on the GPU, map the relevant
        // portion back into CPU-land and run the callback using it.
        let query_state = self.query_state.clone();
        self.select_buf.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            result.expect("map_async failed");
            query_state.replace(SelectManagerQueryState::UnhandledMapped { index, query });
        });
        ctx.device.poll(wgpu::MaintainBase::Poll);

        Ok(())
    }

    /// Checks if there is a mapped query waiting to be picked up from the buffer.
    /// If there is, this function transitions it into the `Mapped` sink state
    /// and performs any select action encoded within it.
    pub fn query_sync(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        let query_state = self.query_state.borrow();
        let SelectManagerQueryState::UnhandledMapped { index, query } = query_state.deref().clone()
        else {
            return;
        };
        drop(query_state);
        ctx.device.poll(wgpu::MaintainBase::WaitForSubmissionIndex(index.clone()));
        self.query_state.replace(SelectManagerQueryState::Mapped(query));
        if matches!(query.action, Some(SelectImmediateAction::Nearest)) {
            state.selection.deselect_all();
        }

        match query.action {
            Some(SelectImmediateAction::Nearest) | Some(SelectImmediateAction::NearestToggle) => {
                let mut nearest: Option<(PixelData, f32)> = None;
                let center_x = (2 * query.rect.x + query.rect.width) as f32 / 2.0;
                let center_y = (2 * query.rect.y + query.rect.height) as f32 / 2.0;
                self.query_use(&query, |(x, y, pixel_data)| {
                    let distance = ((x - center_x).powi(2) + (y - center_y).powi(2)).sqrt();
                    if let Some(nearest) = nearest {
                        if distance >= nearest.1 {
                            return;
                        }
                    }
                    nearest = Some((*pixel_data, distance));
                });
                let Some((pixel_data, _)) = nearest else { return };
                let mesh_id = MeshId::new(pixel_data.mesh_id - 1);
                let vert_id = VertexId::new(pixel_data.el_id);
                if query.action.is_some_and(|action| action == SelectImmediateAction::Nearest) {
                    state.selection.select_verts(&state.meshes[&mesh_id], &[vert_id]);
                } else {
                    state.selection.toggle_verts(&state.meshes[&mesh_id], &[vert_id]);
                }
            }
            Some(SelectImmediateAction::All) => {}
            None => todo!(),
        }
    }

    /// Iterates over select pixels in the supplied rectangle, top-to-left.
    /// If the rect does not fit within the currently-mapped section of the buffer,
    /// or has a different selection mask applied, this function will panic.
    pub fn query_use<F: FnMut((f32, f32, &PixelData))>(&self, query: &SelectionQuery, cb: F) {
        let query_state = self.query_state.borrow();
        let SelectManagerQueryState::Mapped(curr_query) = query_state.deref() else {
            panic!("Attempted to read pixels in unmapped select buffer")
        };
        if !curr_query.contains(query) {
            panic!("Desired query does not match mapped query")
        }
        let tex_width = self.textures.idx.texture.width();
        let tex_block_size = self.textures.block_size;
        let start_idx = (query.rect.y * tex_width + query.rect.x) * tex_block_size;
        let end_idx =
            ((query.rect.y + query.rect.height) * tex_width + query.rect.x + query.rect.width)
                * tex_block_size;
        self.select_buf
            .slice((start_idx as u64)..(end_idx as u64))
            .get_mapped_range()
            .chunks_exact(tex_block_size as usize)
            .zip(0u32..)
            .filter_map(move |(chunk, i)| {
                let pixel_idx = start_idx / tex_block_size + i;
                let x = pixel_idx % tex_width;
                let y = pixel_idx / tex_width;
                let pixel_data = bytemuck::from_bytes::<PixelData>(chunk);
                if pixel_data.mesh_id != 0 // Mesh indices are offset by 1 for valid elements
                    && x >= query.rect.x
                    && y >= query.rect.y
                    && x < query.rect.x + query.rect.width
                    && y < query.rect.y + query.rect.height
                {
                    Some((x as f32, y as f32, pixel_data))
                } else {
                    None
                }
            })
            .for_each(cb)
    }
}

#[derive(Debug)]
struct SelectManagerAttachmentTextures {
    // Object picking / select textures
    idx: gpu::Texture,
    depth: gpu::Texture,

    // The number of bytes in each pixel
    pub block_size: u32,
}

/// The format of the selection idx texture.
pub const TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

/// The actual data stored in each pixel
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PixelData {
    pub _a: u32,
    pub _b: u32,
    pub mesh_id: u32,
    pub el_id: u32,
}

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
                    format: TEX_FORMAT,
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
            block_size,
        }
    }

    /// Returns a corresponding buf to be copied into from the texture
    fn get_buf(&self, ctx: &gpu::Context) -> wgpu::Buffer {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("select.idx_buf"),
            size: (self.depth.texture.width() * self.depth.texture.height() * self.block_size)
                .into(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }
}
