use crate::cache;
use crate::gpu;

pub struct ProgramEdge {
    pipeline: wgpu::RenderPipeline,
}

impl ProgramEdge {
    pub fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/edge.wgsl"));
        let render_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ink3.edge"),
            layout: Some(&ctx.shared_layouts.pipelines.pipeline_3d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_EDIT_EDGES,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
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
        });

        Self { pipeline: render_pipeline }
    }

    /// Writes geometry draw commands for all the materials in a mesh
    pub fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU) {
        // Set the pipeline and draw the mesh
        // TODO: For each material...
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_edit_edges(render_pass);
    }
}