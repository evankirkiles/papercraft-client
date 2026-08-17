use crate::{cache, gpu};

#[derive(Debug)]
pub struct GridCircleProgram {
    pipeline: wgpu::RenderPipeline,
}

impl GridCircleProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader =
            ctx.device.create_shader_module(wgpu::include_wgsl!("./shaders/grid_circle.wgsl"));
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("overlay.grid_circle"),
                layout: Some(&ctx.shared.pipeline_layouts.grid_and_bounds),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: wgpu::VertexFormat::Float32x2.size(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.view_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: (&ctx.settings.msaa_level).into(),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    // A ground plane seen at a grazing angle is the one place
                    // slope-scaled offset earns its keep; the constant term is a
                    // no-op on a float depth target, so it's gone.
                    bias: wgpu::DepthBiasState { slope_scale: 0.9, ..Default::default() },
                }),
                multiview: None,
                cache: None,
            }),
        }
    }

    /// Draws the grid, fit to the document's current bounds.
    pub fn draw(
        &self,
        ctx: &gpu::Context,
        draw_cache: &cache::DrawCache,
        render_pass: &mut wgpu::RenderPass,
    ) {
        draw_cache.bounds.bind(render_pass);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, ctx.shared.buffers.rect.slice(..));
        render_pass.draw(0..4, 0..1);
    }
}
