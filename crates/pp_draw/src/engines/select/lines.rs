use crate::cache;
use crate::gpu;
use crate::select;

#[derive(Debug)]
pub struct LinesProgram {
    pipeline: wgpu::RenderPipeline,
}

impl LinesProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader =
            ctx.device.create_shader_module(wgpu::include_wgsl!("../ink/shaders/lines.wgsl"));
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("select.lines"),
                layout: Some(&ctx.shared.pipeline_layouts.mesh_overlays),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_EDIT_LINES_INSTANCED,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_select"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: select::SELECT_TEX_FORMAT,
                        blend: None,
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
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
        }
    }

    /// Writes geometry draw commands for all the materials in a mesh
    pub(super) fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_edit_lines_instanced(ctx, render_pass);
    }

    /// Writes geometry draw commands for all the materials in a mesh
    pub(super) fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_piece_edit_lines_instanced(ctx, render_pass);
    }
}
