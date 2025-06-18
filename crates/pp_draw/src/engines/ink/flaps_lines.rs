use crate::cache;
use crate::gpu;

#[derive(Debug)]
pub(super) struct FlapsLinesProgram {
    pipeline: wgpu::RenderPipeline,
}

impl FlapsLinesProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("../shaders/flaps.wgsl"));
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ink3.flaps_lines"),
                layout: Some(&ctx.shared_layouts.pipelines.pipeline_3d),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_edge"),
                    buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_FLAPS_INSTANCED,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.view_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: super::DepthBiasLayer::BackgroundTop as i32,
                        slope_scale: 0.03,
                        ..Default::default()
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

    /// Writes geometry draw commands for all the materials in a mesh
    pub(super) fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_piece_flaps_outline_instanced(ctx, render_pass);
    }
}
