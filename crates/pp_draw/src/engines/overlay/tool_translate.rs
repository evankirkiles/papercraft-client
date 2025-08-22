use crate::cache::tool::translate::TranslateToolGPU;
use crate::engines::ink::DepthBiasLayer;
use crate::gpu;

#[derive(Debug)]
pub struct ToolTranslateProgram {
    pipeline: wgpu::RenderPipeline,
}

impl ToolTranslateProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader =
            ctx.device.create_shader_module(wgpu::include_wgsl!("./shaders/tool_translate.wgsl"));
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("tool.translate"),
                layout: Some(&ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("tool.translate"),
                    bind_group_layouts: &[
                        &ctx.shared.bind_group_layouts.settings,
                        &ctx.shared.bind_group_layouts.viewport,
                        &ctx.shared.bind_group_layouts.tool.translate,
                    ],
                    push_constant_ranges: &[],
                })),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: {
                        wgpu::DepthBiasState {
                            constant: DepthBiasLayer::BackgroundBottom as i32,
                            ..Default::default()
                        }
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: (&ctx.settings.msaa_level).into(),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
        }
    }

    pub(super) fn draw(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        tool: &TranslateToolGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, ctx.shared.buffers.rect.slice(..));
        tool.bind(render_pass);
        render_pass.draw(0..4, 0..1);
    }
}
