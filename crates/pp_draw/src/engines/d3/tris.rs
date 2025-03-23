use crate::cache;
use crate::engines::program;
use crate::gpu;

#[derive(Debug)]
pub struct Program {
    surface_pipeline: wgpu::RenderPipeline,
    surface_pipeline_xray: wgpu::RenderPipeline,
}

impl program::MeshDrawable for Program {
    fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("../shaders/tris.wgsl"));
        let layout = Some(&ctx.shared_layouts.pipelines.pipeline_3d);
        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: cache::MeshGPU::BATCH_BUFFER_LAYOUT_SURFACE,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };
        let targets = [Some(wgpu::ColorTargetState {
            format: ctx.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let fragment = Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
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
        let multiview = None;
        let cache = None;

        let surface_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ink3.surface"),
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                // Apply a tiny depth bias to reduce flickering of lines at same depth
                bias: wgpu::DepthBiasState { constant: 1, slope_scale: 0.05, ..Default::default() },
            }),
        });

        let surface_pipeline_xray =
            ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ink3.surface"),
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
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    // Apply a tiny depth bias to reduce flickering of lines at same depth
                    bias: wgpu::DepthBiasState {
                        constant: 1,
                        slope_scale: 0.05,
                        ..Default::default()
                    },
                }),
            });

        Self { surface_pipeline, surface_pipeline_xray }
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

impl Program {
    pub fn draw_mesh_xrayed(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        render_pass.set_pipeline(&self.surface_pipeline_xray);
        mesh.draw_surface(render_pass);
    }
}
