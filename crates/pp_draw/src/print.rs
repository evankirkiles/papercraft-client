//! Rasterizing the print layout's pages to images.
//!
//! Printing is the same draw the cutting viewport makes - the pieces, their
//! fold and cut lines, and their flaps - with three substitutions: a camera
//! framing exactly one sheet instead of the user's pan/zoom, an offscreen
//! target sized to the sheet's physical dimensions at a print resolution
//! instead of the swapchain, and none of the scene extras that only make sense
//! on screen (the grid, the page backdrop, the margins, the active tool).
//!
//! The readback follows the same shape as [`crate::select`]: render, copy the
//! texture into a mappable buffer, and poll for the mapping to complete rather
//! than blocking, since the browser's event loop is the only thing that can
//! drive it forward.

use std::{cell::RefCell, io::Cursor, iter, rc::Rc};

use image::ImageEncoder;
use pp_core::{
    measures::{Dimensions, Rect},
    print::PageSize,
};
use pp_editor::{
    preferences::theme::Theme, state::SelectionMode,
    viewport::camera::orthographic::OrthographicCamera,
};

use crate::{
    cache::{
        settings::{SettingsGPU, ThemeOverrides},
        viewport::{bounds::ViewportBoundsUniform, camera::CameraUniform},
    },
    engines::ink::InkEngine,
    gpu::{self, shared::bind_group_layouts::BindGroup},
    Renderer,
};

/// The resolution a print page is rasterized at, if the caller doesn't say.
pub const DEFAULT_PRINT_DPI: f32 = 300.0;

/// The pixel density print strokes are sized against.
///
/// The theme's widths are authored in screen pixels, where a line has to
/// survive a coarse ~96 DPI grid to stay visible at all. Scaling them by that
/// same ratio prints them far heavier than they look - a 4px cut line would
/// come out at over a millimeter. Sizing against 254 DPI instead makes one
/// theme pixel a tenth of a millimeter on paper, so the thick strokes land
/// near 0.4mm: crisp, and thin enough to cut along accurately.
const PRINT_STROKE_REFERENCE_DPI: f32 = 254.0;

/// Print renders without MSAA. A pixel at print resolution is already far finer
/// than antialiasing could usefully resolve - at 300 DPI it is 0.085mm, below
/// what the eye or an inkjet's dot gain distinguishes - and the 4x attachments
/// an A4 sheet would need run to hundreds of megabytes.
const PRINT_SAMPLE_COUNT: u32 = 1;

#[derive(Debug)]
pub enum PrintError {
    /// The requested resolution needs a texture larger than the device allows.
    PageTooLarge { width: u32, height: u32, max: u32 },
    /// The GPU dropped the readback buffer before it could be mapped.
    ReadbackFailed,
    /// The pixels came back but couldn't be encoded as a PNG.
    EncodeFailed(String),
}

impl std::fmt::Display for PrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrintError::PageTooLarge { width, height, max } => write!(
                f,
                "a {width}x{height} page exceeds this device's maximum texture size of {max}"
            ),
            PrintError::ReadbackFailed => write!(f, "reading the rendered page back failed"),
            PrintError::EncodeFailed(err) => write!(f, "encoding the page as a PNG failed: {err}"),
        }
    }
}

/// Where a page render has got to. Mirrors [`crate::select`]'s query states.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PageState {
    /// Nothing submitted; the readback buffer is free.
    Idle,
    /// Draw calls are in flight on the GPU.
    Rendering,
    /// The copy landed and the buffer is being mapped into CPU memory.
    Mapping,
    /// The buffer is mapped and its pixels are ready to read.
    Mapped,
}

/// The result of polling an in-flight page render.
#[derive(Debug)]
pub enum PrintPoll {
    /// Nothing has been submitted.
    Idle,
    /// Still waiting on the GPU.
    Pending,
    /// The page's PNG bytes.
    Ready(Result<Vec<u8>, PrintError>),
}

/// Rounds a number up to the nearest multiple of `align`.
const fn align_up(num: u32, align: u32) -> u32 {
    (num + (align - 1)) & !(align - 1)
}

/// The offscreen target a print run rasterizes into, reused across every page
/// of the run: one sheet at one resolution, rendered and read back one at a
/// time so a large grid doesn't multiply the memory.
#[derive(Debug)]
pub struct PrintTarget {
    /// The page's true size in pixels. The color texture is wider (see
    /// `padded_width`), so this is what the render pass's viewport is set to.
    size: Dimensions<u32>,
    /// The color texture's width, padded so each row of the texture-to-buffer
    /// copy lands on a 256-byte boundary. Nothing rasterizes into the pad
    /// because the viewport is set to `size`, and it is sliced off on readback.
    padded_width: u32,

    color: gpu::Texture,
    depth: gpu::Texture,
    /// Whether the color texture's bytes are laid out BGRA rather than RGBA.
    is_bgra: bool,

    // Rc/RefCell'd because `map_async` takes a 'static callback.
    readback: Rc<RefCell<wgpu::Buffer>>,
    state: Rc<RefCell<PageState>>,

    /// Pipelines built at [`PRINT_SAMPLE_COUNT`], since the on-screen ones bake
    /// in the user's MSAA level.
    engine_ink: InkEngine,
    /// The theme the print pass renders under: strokes scaled to the page's
    /// pixel density, and no selection highlighting.
    settings: SettingsGPU,

    camera_buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
}

impl PrintTarget {
    pub fn new(
        ctx: &gpu::Context,
        page_size: &PageSize,
        theme: &Theme,
        dpi: f32,
    ) -> Result<Self, PrintError> {
        let size = page_size.pixels(dpi);
        let max = ctx.device.limits().max_texture_dimension_2d;

        let format = ctx.config.format;
        let block_size = format.block_copy_size(None).unwrap_or(4);
        // Widen the texture rather than copying row by row with a padded
        // stride, the same trick the select pass uses.
        let padded_width =
            align_up(size.width * block_size, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) / block_size;
        if padded_width > max || size.height > max {
            return Err(PrintError::PageTooLarge { width: padded_width, height: size.height, max });
        }

        let extent =
            wgpu::Extent3d { width: padded_width, height: size.height, depth_or_array_layers: 1 };
        let color_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("print.color"),
            size: extent,
            mip_level_count: 1,
            sample_count: PRINT_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[ctx.view_format],
        });
        // The pipelines' color target is `ctx.view_format`, which on the WebGPU
        // path is the sRGB view of a non-sRGB texture - so shaders write linear
        // values and the hardware encodes them, exactly as it does for the
        // swapchain. The bytes we copy out are therefore already sRGB.
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor {
            format: (ctx.view_format != format).then_some(ctx.view_format),
            ..Default::default()
        });
        let color = gpu::Texture { texture: color_texture, view: color_view };

        let depth = gpu::Texture::new(
            ctx,
            wgpu::TextureDescriptor {
                label: Some("print.depth"),
                size: extent,
                mip_level_count: 1,
                sample_count: PRINT_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: gpu::Texture::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("print.readback"),
            size: (padded_width * size.height * block_size) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // The bind group slots are fixed globally, so the print pass needs a
        // viewport binding even though it has no editor viewport behind it.
        let mut bounds_buf = gpu::UniformBuf::new(
            ctx,
            "print.viewport".to_string(),
            size_of::<ViewportBoundsUniform>(),
        );
        bounds_buf.update(
            ctx,
            &[ViewportBoundsUniform::for_area(Rect {
                x: 0.0,
                y: 0.0,
                width: size.width as f32,
                height: size.height as f32,
            })],
        );
        let camera_buf =
            gpu::UniformBuf::new(ctx, "print.camera".to_string(), size_of::<CameraUniform>());
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("print.viewport"),
            layout: &ctx.shared.bind_group_layouts.viewport,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: bounds_buf.binding_resource() },
                wgpu::BindGroupEntry { binding: 1, resource: camera_buf.binding_resource() },
            ],
        });

        let overrides =
            ThemeOverrides { stroke_scale: dpi / PRINT_STROKE_REFERENCE_DPI, selection: false };
        Ok(Self {
            size,
            padded_width,
            color,
            depth,
            is_bgra: matches!(
                format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
            ),
            readback: Rc::new(RefCell::new(readback)),
            state: Rc::new(RefCell::new(PageState::Idle)),
            engine_ink: InkEngine::new(ctx, PRINT_SAMPLE_COUNT),
            settings: SettingsGPU::new_with_overrides(ctx, theme, &overrides),
            camera_buf,
            bind_group,
        })
    }
}

impl<'window> Renderer<'window> {
    /// Renders one page of the print layout and starts reading it back.
    ///
    /// `page` is the sheet's area in world (centimeter) space. Poll
    /// [`Self::print_poll`] until it yields the encoded PNG; only one page may
    /// be in flight at a time, since they share the readback buffer.
    pub fn print_page(&mut self, target: &mut PrintTarget, page: Rect<f32>) {
        let Renderer { ctx, draw_cache, .. } = &self;

        let camera = OrthographicCamera::framing(&page);
        let area: Rect<f32> =
            Dimensions { width: target.size.width as f32, height: target.size.height as f32 }
                .into();
        target.camera_buf.update(ctx, &[CameraUniform::new(&camera, area)]);

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("print") });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("print"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.color.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Transparent, so only the pieces themselves carry any
                        // ink - the sheet is the paper the user prints on.
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // Confine rasterization to the page, leaving the row-alignment pad
            // on the right untouched.
            render_pass.set_viewport(
                0.0,
                0.0,
                target.size.width as f32,
                target.size.height as f32,
                0.0,
                1.0,
            );
            target.settings.bind(&mut render_pass);
            render_pass.set_bind_group(BindGroup::Viewport.value(), &target.bind_group, &[]);
            // Piece mode is what the printout is: fold annotations rather than
            // the editable wireframe, and no vertex handles.
            Self::draw_pieces(
                ctx,
                draw_cache,
                &target.engine_ink,
                &SelectionMode::Piece,
                &mut render_pass,
            );
        }

        let block_size = ctx.config.format.block_copy_size(None).unwrap_or(4);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &target.color.texture,
                origin: wgpu::Origin3d::ZERO,
                mip_level: 0,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &target.readback.borrow(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(target.padded_width * block_size),
                    rows_per_image: Some(target.size.height),
                },
            },
            wgpu::Extent3d {
                width: target.padded_width,
                height: target.size.height,
                depth_or_array_layers: 1,
            },
        );
        ctx.queue.submit(iter::once(encoder.finish()));
        target.state.replace(PageState::Rendering);
    }

    /// Advances the in-flight page render, yielding its PNG once it lands.
    ///
    /// Like the select readback this never blocks: on the web the only thing
    /// that can complete a buffer mapping is the browser's event loop, so this
    /// is expected to be called once a frame.
    pub fn print_poll(&mut self, target: &mut PrintTarget) -> PrintPoll {
        let state = *target.state.borrow();
        match state {
            PageState::Idle => PrintPoll::Idle,
            PageState::Rendering => {
                if self.ctx.device.poll(wgpu::MaintainBase::Poll).is_ok_and(|f| f.is_queue_empty())
                {
                    target.state.replace(PageState::Mapping);
                    let state = target.state.clone();
                    target.readback.borrow().slice(..).map_async(
                        wgpu::MapMode::Read,
                        move |result| {
                            // A failed mapping leaves the state at Mapping; the
                            // caller times out rather than hanging on a buffer
                            // that will never arrive.
                            if result.is_ok() {
                                state.replace(PageState::Mapped);
                            }
                        },
                    );
                }
                PrintPoll::Pending
            }
            PageState::Mapping => PrintPoll::Pending,
            PageState::Mapped => {
                let png = {
                    let readback = target.readback.borrow();
                    let mapped = readback.slice(..).get_mapped_range();
                    encode_png(&mapped, target.size, target.padded_width, target.is_bgra)
                };
                target.readback.borrow().unmap();
                target.state.replace(PageState::Idle);
                PrintPoll::Ready(png)
            }
        }
    }
}

/// Turns a mapped page readback into PNG bytes.
///
/// Two corrections stand between the raw texels and a usable image:
///
/// - The texture is padded on the right for the copy's row alignment, so each
///   row has to be cut back to the page's true width.
/// - Every annotation pipeline blends with [`wgpu::BlendState::ALPHA_BLENDING`]
///   over a transparent black clear, which leaves the result *premultiplied*
///   (`rgb = a * color`). PNG stores straight alpha, so without dividing it
///   back out every antialiased stroke prints with a dark halo.
fn encode_png(
    bytes: &[u8],
    size: Dimensions<u32>,
    padded_width: u32,
    is_bgra: bool,
) -> Result<Vec<u8>, PrintError> {
    let row_stride = (padded_width * 4) as usize;
    let mut pixels = Vec::with_capacity((size.width * size.height * 4) as usize);
    for row in 0..size.height as usize {
        let start = row * row_stride;
        let row = &bytes[start..start + (size.width * 4) as usize];
        for texel in row.chunks_exact(4) {
            let (r, g, b) = if is_bgra {
                (texel[2], texel[1], texel[0])
            } else {
                (texel[0], texel[1], texel[2])
            };
            let a = texel[3];
            let straight = |c: u8| -> u8 {
                if a == 0 {
                    0
                } else {
                    ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8
                }
            };
            pixels.extend_from_slice(&[straight(r), straight(g), straight(b), a]);
        }
    }

    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(Cursor::new(&mut png))
        .write_image(&pixels, size.width, size.height, image::ExtendedColorType::Rgba8)
        .map_err(|err| PrintError::EncodeFailed(err.to_string()))?;
    Ok(png)
}
