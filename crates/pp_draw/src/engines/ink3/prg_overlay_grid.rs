use crate::cache;
use crate::gpu;

pub struct ProgramOverlayGrid {
    pipeline: wgpu::RenderPipeline,
}

impl ProgramOverlayGrid {
    pub fn new(ctx: &gpu::Context) -> Self {
        let shader =
            ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/overlay_grid.wgsl"));
        let render_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ink3.overlay_grid"),
            layout: Some(&ctx.shared_layouts.pipelines.pipeline_3d_deferred),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
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
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        Self { pipeline: render_pipeline }
    }

    /// Draws the grid (only done once)
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..4, 0..1);
    }
}
