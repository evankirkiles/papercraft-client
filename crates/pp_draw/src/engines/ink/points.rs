use crate::cache;
use crate::gpu;

#[derive(Debug)]
pub struct PointsProgram {
    pipeline: wgpu::RenderPipeline,
    pipeline_xray: wgpu::RenderPipeline,
}

impl PointsProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("../shaders/points.wgsl"));
        let layout = Some(&ctx.shared_layouts.pipelines.pipeline_3d);
        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_EDIT_POINTS_INSTANCED,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };
        let targets = [Some(wgpu::ColorTargetState {
            format: ctx.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let fragment = Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
        let multisample = wgpu::MultisampleState {
            count: (&ctx.settings.msaa_level).into(),
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let bias = wgpu::DepthBiasState {
            constant: super::DepthBiasLayer::ForegroundTop as i32,
            ..Default::default()
        };
        let multiview = None;
        let cache = None;

        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ink3.points"),
                vertex: vertex.clone(),
                fragment: fragment.clone(),
                layout,
                primitive,
                multisample,
                multiview,
                cache,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias,
                }),
            }),
            pipeline_xray: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ink3.points.xray"),
                vertex,
                fragment,
                layout,
                primitive,
                multisample,
                multiview,
                cache,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: wgpu::StencilState::default(),
                    bias,
                }),
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
        mesh.draw_edit_points_instanced(ctx, render_pass);
    }

    pub(super) fn draw_mesh_xrayed(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline_xray);
        mesh.draw_edit_points_instanced(ctx, render_pass);
    }

    pub(super) fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_piece_edit_points_instanced(ctx, render_pass);
    }
}
