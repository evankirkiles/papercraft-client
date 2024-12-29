use crate::cache;
use crate::gpu;

pub struct WireEdgeModule {
    pipeline: wgpu::RenderPipeline,
}

impl WireEdgeModule {
    pub fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/wire_edge.wgsl"));
        let render_pipeline_layout =
            ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Wire Edge Pipeline Layout"),
                bind_group_layouts: &[&ctx.shared_bind_group_layouts.camera],
                push_constant_ranges: &[],
            });
        let render_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wire Edge Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: cache::batch_buffer_layouts::EDIT_EDGES,
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
        mesh.batches.edit_edges.bind(render_pass);
        mesh.batches.edit_edges.draw_indexed(render_pass);

        // POTENTIALLY OUTDATED:
        // Resource manager:
        //  1. Given mesh, fetch material batches (VBO, IBO, Tex) of tri sets
        //  2. When untextured, batch is just the entire IBO

        // 0. SETUP
        // Begin by binding the mesh's VBO. This won't change throughout draw calls
        // Then, run through the following pipelines
        // render_pass.set_vertex_buffer(1, mesh.vertex_buffer.slice(..));

        // 1. (SUR)FACE CALL
        // Bind face selection state buffer (used in frag shader)
        // For material in mesh
        //  |- Bind TRILIST material's batch (IBOs & Texture)
        //  |- Run draw calls to draw textured faces for batch

        // for batch in mesh.batches.iter() {
        //     render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        //     render_pass.set_index_buffer(
        //         batch.index_buffer.slice(..),
        //         wgpu::IndexFormat::Uint32,
        //     );
        //     render_pass.draw_indexed(0..batch.len, 0, 0..1);
        // }

        // 2. EDGE CALL
        // Bind vert selection state buffer (used in vert shader)
        // Bind model's LINELIST  batch
        //  |- Run draw call to draw edges with gradient for selected state
        //
        // 3. VERTEX CALL (optional)
        // Bind vert selection state buffer (used in vert shader)
        // Bind model's VERTLIST  batch
        //  |- Run draw call to draw vertices
        //
    }
}
