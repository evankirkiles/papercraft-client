use crate::{
    cache::print::{PageVertexAttributes, PrintLayoutGPU},
    engines::ink::DepthBiasLayer,
    gpu,
};

#[derive(Debug)]
pub struct PageProgram {
    pipeline: wgpu::RenderPipeline,
    pipeline_margins: wgpu::RenderPipeline,
}

impl PageProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("./shaders/page.wgsl"));
        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[
                wgpu::VertexBufferLayout {
                    step_mode: wgpu::VertexStepMode::Instance,
                    array_stride: std::mem::size_of::<PageVertexAttributes>() as u64,
                    attributes: &PageVertexAttributes::vertex_attributes(0),
                },
                wgpu::VertexBufferLayout {
                    array_stride: wgpu::VertexFormat::Float32x2.size(),
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 1,
                    }],
                },
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };
        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: ctx.view_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        };
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("page"),
            bind_group_layouts: &[
                &ctx.shared.bind_group_layouts.settings,
                &ctx.shared.bind_group_layouts.viewport,
                &ctx.shared.bind_group_layouts.print_layout,
            ],
            push_constant_ranges: &[],
        });
        let descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("page"),
            layout: Some(&layout),
            vertex: vertex_state.clone(),
            fragment: Some(fragment_state.clone()),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: gpu::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: DepthBiasLayer::BackgroundBottom as i32,
                    ..Default::default()
                },
            }),
            multiview: None,
            cache: None,
        };

        Self {
            pipeline: ctx.device.create_render_pipeline(&descriptor.clone()),
            pipeline_margins: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                vertex: wgpu::VertexState { entry_point: Some("vs_margins"), ..vertex_state },
                fragment: Some(wgpu::FragmentState {
                    entry_point: Some("fs_margins"),
                    ..fragment_state
                }),
                ..descriptor
            }),
        }
    }

    pub fn draw(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        print: &PrintLayoutGPU,
    ) {
        if print.pages.len == 0 {
            return;
        }
        print.bind(render_pass);
        render_pass.set_vertex_buffer(0, print.pages.slice());
        render_pass.set_vertex_buffer(1, ctx.shared.buffers.rect.slice(..));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..4, 0..print.pages.len);
        render_pass.set_vertex_buffer(1, ctx.shared.buffers.rect_outline.slice(..));
        render_pass.set_pipeline(&self.pipeline_margins);
        render_pass.draw(0..24, 0..print.pages.len);
    }
}
