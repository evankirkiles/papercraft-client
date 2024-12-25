use wgpu::util::RenderEncoder;

use crate::gpu;

pub struct GeometryModule {
    surface_pipeline: wgpu::RenderPipeline,
}

impl GeometryModule {
    pub fn new(ctx: &gpu::GPUContext) -> Self {
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
        let render_pipeline_layout = ctx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Pipeline Layout"),
                bind_group_layouts: &[], // TODO: These Bind groups
                push_constant_ranges: &[],
            },
        );
        let render_pipeline = ctx.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Mesh Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[], // TODO: These buffers
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::GPUTexture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
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
            },
        );

        Self {
            surface_pipeline: render_pipeline,
        }
    }

    // TODO: Prepares all buffers and things needed for drawing
    pub fn begin_sync(&self) {}

    /// Writes geometry draw commands for all the materials in a mesh
    pub fn sync_model(
        &self,
        render_pass: &mut wgpu::RenderPass,
        // mesh: &resources::GPUModel,
    ) {
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
        render_pass.set_pipeline(&self.surface_pipeline);
        // TODO: Remove this
        render_pass.draw(0..3, 0..1);
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

    // TODO: Cleans up all buffers and things needed for drawing
    pub fn end_sync(&self) {}
}
