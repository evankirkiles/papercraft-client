use crate::cache;
use crate::engines::program::MeshDrawable;
use crate::gpu;

pub struct Program {
    surface_pipeline: wgpu::RenderPipeline,
}

impl MeshDrawable for Program {
    fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("../shaders/tris.wgsl"));
        let render_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ink3.surface"),
            layout: Some(&ctx.shared_layouts.pipelines.pipeline_3d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_SURFACE,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                // Apply a tiny depth bias to reduce flickering of lines at same depth
                bias: wgpu::DepthBiasState { constant: 1, slope_scale: 0.05, ..Default::default() },
            }),
            multisample: wgpu::MultisampleState {
                count: (&ctx.settings.msaa_level).into(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self { surface_pipeline: render_pipeline }
    }

    /// Writes geometry draw commands for all the materials in a mesh
    fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.surface_pipeline);
        mesh.draw_surface(render_pass);
    }
}
