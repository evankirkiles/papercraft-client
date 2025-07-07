use bitflags::bitflags;
use cgmath::Point2;
use pp_core::measures::Rect;
use std::fmt::Debug;
use std::{cell::RefCell, iter, ops::Deref, rc::Rc};

use crate::cache;
use crate::cache::viewport::BindableViewport;
use crate::engines::select::SelectEngine;
use crate::gpu;

bitflags! {
    /// A mask of items to render for selection in the buffer
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct SelectionMask: u8 {
        const VERTS = 1 << 0;
        const EDGES = 1 << 1;
        const FACES = 1 << 2;
        const PIECES = 1 << 3;
    }
}

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

#[derive(Debug, Copy, Clone)]
pub struct SelectionQueryArea {
    pub rect: Rect<u32>,
    pub mask: SelectionMask,
}

pub type SelectionPixelData = Vec<(Point2<f32>, PixelData)>;

/// A single query submitted to the GPU to populate the
pub struct SelectionQuery<'a> {
    pub area: SelectionQueryArea,
    pub callback: Option<&'a mut dyn Fn(&SelectionPixelData)>,
}

impl<'a> Debug for SelectionQuery<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionQuery").field("area", &self.area).finish()
    }
}

impl SelectionQueryArea {
    fn contains(&self, other: &SelectionQueryArea) -> bool {
        self.mask.contains(other.mask) && self.rect.contains_rect(&other.rect)
    }
}

/// The final results of a GPU selection query
#[derive(Debug, Clone)]
pub struct SelectionQueryResult {
    pub area: SelectionQueryArea,
    pub pixels: SelectionPixelData,
}

#[derive(Debug)]
pub enum SelectionQueryError {
    QueryInFlight,
}

/// Represents whether a selection query is currently being processed.
#[derive(Debug, Clone)]
enum SelectManagerQueryState {
    /// No selection query has been performed, and the CPU-side buffer is unmapped
    Unmapped,
    /// A selection query was submitted to the GPU, but is not yet ready
    QueryingGPU(SelectionQueryArea),
    /// The query from the GPU is ready, but the CPU-side buffer is not yet mapped
    MappingToCPU(SelectionQueryArea),
    /// The CPU-side buffer is mapped
    Mapped(SelectionQueryResult),
}

pub type SelectionCallback = dyn Fn(&SelectionQueryArea, &SelectionQueryResult);
pub(super) struct SelectManager {
    /// The rendering engine for drawing into the select textures
    pub(crate) select_engine: SelectEngine,
    /// The GPU textures the select engine renders into
    textures: SelectManagerAttachmentTextures,

    // The below variables are Rc/RefCell'ed because we need to pass them into
    // a 'static asynchronous callback for wgpu::Buffer's `map_async` function
    /// The GPU buffer the texture is copied into, later maps to a CPU buffer
    select_buf: Rc<RefCell<wgpu::Buffer>>,
    /// The state of reading back from the buffer
    query_state: Rc<RefCell<SelectManagerQueryState>>,
    /// Callbacks to execute once the selection query completes
    query_callbacks: Vec<(SelectionQueryArea, Box<SelectionCallback>)>,
}

impl Debug for SelectManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectManager")
            .field("select_engine", &self.select_engine)
            .field("textures", &self.textures)
            .field("select_buf", &self.select_buf)
            .field("query_state", &self.query_state)
            .finish()
    }
}

impl SelectManager {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let textures = SelectManagerAttachmentTextures::create(ctx);
        let select_buf = textures.get_buf(ctx);
        Self {
            select_engine: SelectEngine::new(ctx),
            textures,
            select_buf: Rc::new(RefCell::new(select_buf)),
            query_state: Rc::new(RefCell::new(SelectManagerQueryState::Unmapped)),
            query_callbacks: Vec::new(),
        }
    }

    /// Queries for `ID`s of element drawn within the supplied rect
    ///
    /// Note that this depends on a asynchronous process of draw calls, texture to
    /// buffer copying, and then buffer mapping to get the data from the GPU to
    /// the CPU. For this reason, you must pass your operation as a command
    /// alongside an event loop - once the GPU marks the buffer as mapped and
    /// ready to be used, an event will be emitted onto the event loop for
    /// further processing.
    pub(crate) fn query<F: Fn(&SelectionQueryArea, &SelectionQueryResult) + 'static>(
        &mut self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        area: SelectionQueryArea,
        callback: Box<F>,
    ) -> Result<(), SelectionQueryError> {
        let query_state = self.query_state.borrow().clone();
        match query_state.clone() {
            // If select buffer is unmapped, we're free to use it
            SelectManagerQueryState::Unmapped => {}
            // If the select buffer has already been queried, we can try to queue
            // the callback up for execution if it's within the area. Otherwise,
            // we can't really do anything, so Error out.
            SelectManagerQueryState::QueryingGPU(curr_area)
            | SelectManagerQueryState::MappingToCPU(curr_area) => {
                if curr_area.contains(&area) {
                    self.query_callbacks.push((area, callback));
                    return Ok(());
                }
                return Err(SelectionQueryError::QueryInFlight);
            }
            // If select buffer is mapped, check if it already contains our results.
            // If yes, we can just re-use the mapped region of the buffer and not
            // have to take the hit of another GPU query.
            // TODO: Proper mapped buffer invalidation on scene change
            SelectManagerQueryState::Mapped(SelectionQueryResult { area: curr_area, .. }) => {
                if curr_area.contains(&area) {
                    // "Poll" synchronously to immediately execute the callback
                    self.query_callbacks.push((area, callback));
                    self.poll(ctx);
                    return Ok(());
                } else {
                    self.select_buf.borrow().unmap();
                }
            }
        };

        // At this point, we know we need to re-query the GPU for a new selection buffer.
        // Begin render pass to draw into the selection buffer
        self.query_callbacks.push((area, callback));
        let SelectionQueryArea { rect, mask } = area;
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
            draw_cache.viewports.iter().for_each(|(_, viewport)| {
                use cache::viewport::ViewportGPU;
                viewport.bind(&mut render_pass);
                match viewport {
                    ViewportGPU::Folding(_) => {
                        self.draw_folding(ctx, draw_cache, mask, &mut render_pass)
                    }
                    ViewportGPU::Cutting(_) => {
                        self.draw_cutting(ctx, draw_cache, mask, &mut render_pass)
                    }
                }
            });
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
                buffer: &self.select_buf.borrow(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: (rect.y * bytes_per_row + rect.x * block_size).into(),
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.textures.idx.texture.height()),
                },
            },
            wgpu::Extent3d { width: rect.width, height: rect.height, depth_or_array_layers: 1 },
        );

        // Submit all commands and track the SubmissionIndex for completion
        ctx.queue.submit(iter::once(encoder.finish()));
        self.query_state.replace(SelectManagerQueryState::QueryingGPU(area));
        Ok(())
    }

    /// Checks if there is a mapped query waiting to be picked up from the buffer.
    /// If there is, this function transitions it into the `Mapped` sink state
    /// and performs any select action encoded within it.
    pub(super) fn poll(&mut self, ctx: &gpu::Context) {
        // Borrow some RcRefCell values we're going to need to pass to a 'static `map_async` callback
        let query_state = self.query_state.borrow();
        let curr_state = query_state.clone();
        drop(query_state);
        match curr_state {
            // If the query is ready and we have callbacks which haven't run,
            // run them and clear out the rest of the callbacks.
            SelectManagerQueryState::Mapped(result) => {
                if !self.query_callbacks.is_empty() {
                    self.query_callbacks
                        .drain(..)
                        .for_each(|(area, callback)| callback(&area, &result));
                }
            }
            // If a selection query submission is in-flight, wait for the queue
            // of submissions to be empty ( indicating it completed ). Once that
            // happens, request the buffer be mapped into the CPU.
            SelectManagerQueryState::QueryingGPU(area) => {
                if ctx.device.poll(wgpu::MaintainBase::Poll).is_ok_and(|f| f.is_queue_empty()) {
                    self.query_state.replace(SelectManagerQueryState::MappingToCPU(area));
                    let tex_width = self.textures.idx.texture.width();
                    let tex_block_size = self.textures.block_size;
                    let query_state = self.query_state.clone();
                    let select_buf = self.select_buf.clone();
                    self.select_buf.borrow().slice(..).map_async(wgpu::MapMode::Read, move |_| {
                        // On successful CPU-side mapping, calculate the final pixel data
                        let start_idx = (area.rect.y * tex_width + area.rect.x) * tex_block_size;
                        let end_idx = ((area.rect.y + area.rect.height) * tex_width
                            + area.rect.x
                            + area.rect.width)
                            * tex_block_size;
                        let pixels: Vec<_> = select_buf
                            .borrow()
                            .slice((start_idx as u64)..(end_idx as u64))
                            .get_mapped_range()
                            .chunks_exact(tex_block_size as usize)
                            .zip(0u32..)
                            .filter_map(move |(chunk, i)| {
                                let pixel_idx = start_idx / tex_block_size + i;
                                let pos =
                                    Point2 { x: pixel_idx % tex_width, y: pixel_idx / tex_width };
                                let pixel_data = bytemuck::from_bytes::<PixelData>(chunk);
                                // Mesh indices are offset by 1 for valid elements
                                (pixel_data.mesh_id != 0 && area.rect.contains(&pos)).then_some((
                                    Point2 { x: pos.x as f32, y: pos.y as f32 },
                                    *pixel_data,
                                ))
                            })
                            .collect();
                        query_state.replace(SelectManagerQueryState::Mapped(
                            SelectionQueryResult { area, pixels },
                        ));
                    });
                }
            }
            // If we're not expecting anything from the GPU or are waiting
            // on our CPU-side buffer to be mapped, just return.
            _ => {}
        }
    }

    /// Updates the GPUContext for new dimensions
    pub(super) fn resize(&mut self, ctx: &gpu::Context) {
        // Block until any in-flight selection queries have been processed, as
        // they expect a corresponding buffer to map into (which we are recreating)
        match self.query_state.borrow().deref() {
            SelectManagerQueryState::Unmapped => {}
            SelectManagerQueryState::QueryingGPU(_) | SelectManagerQueryState::MappingToCPU(_) => {
                while !ctx.device.poll(wgpu::MaintainBase::Wait).is_ok_and(|f| f.is_queue_empty()) {
                }
            }
            SelectManagerQueryState::Mapped(_) => {
                self.select_buf.borrow().unmap();
            }
        }
        self.query_state.replace(SelectManagerQueryState::Unmapped);
        self.textures = SelectManagerAttachmentTextures::create(ctx);
        self.select_buf.replace(self.textures.get_buf(ctx));
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
pub(crate) const SELECT_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

/// The actual data stored in each pixel
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PixelData {
    pub f_id: u32,    // (G) Face Id
    pub el_id: u32,   // (B) Edge / Vertex ID
    pub mesh_id: u64, // (A) Mesh ID
}

/// Rounds a number up to the nearest multiple of `align`
const fn align_up(num: u32, align: u32) -> u32 {
    ((num) + ((align) - 1)) & !((align) - 1)
}

impl SelectManagerAttachmentTextures {
    fn create(ctx: &gpu::Context) -> Self {
        // Align the width of the image up to a 256-byte alignment per row, as
        // required to use `copy_texture_to_buffer`. This will not affect the
        // final image, as we always set the viewport before rendering.
        let block_size = SELECT_TEX_FORMAT.block_copy_size(None).unwrap();
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
                    format: SELECT_TEX_FORMAT,
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
