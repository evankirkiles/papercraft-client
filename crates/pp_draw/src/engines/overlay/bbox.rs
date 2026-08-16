use pp_core::bounds::Aabb3;

use crate::{engines::ink::DepthBiasLayer, gpu};

/// One edge of the wireframe box: its two world-space endpoints.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct EdgeInstance {
    v0: [f32; 3],
    v1: [f32; 3],
}

/// Draws a wireframe box around the document's world-space bounds in the
/// folding viewport, using the same screen-space-thickness instanced-line
/// technique as `engines::ink::lines`, simplified for world-space endpoints
/// (no per-mesh piece transform, no per-vertex flags/select index).
#[derive(Debug)]
pub struct BboxProgram {
    pipeline: wgpu::RenderPipeline,
    instances: gpu::VertBuf,
    last: Option<Aabb3>,
}

impl BboxProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("./shaders/bbox.wgsl"));
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay.bbox"),
            bind_group_layouts: &[
                &ctx.shared.bind_group_layouts.settings,
                &ctx.shared.bind_group_layouts.viewport,
            ],
            push_constant_ranges: &[],
        });
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("overlay.bbox"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: wgpu::VertexFormat::Float32x2.size(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<EdgeInstance>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 1,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: wgpu::VertexFormat::Float32x3.size(),
                                    shader_location: 2,
                                },
                            ],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.view_format,
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gpu::Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: DepthBiasLayer::ForegroundBottom as i32,
                        slope_scale: 0.03,
                        ..Default::default()
                    },
                }),
                multiview: None,
                cache: None,
            }),
            instances: gpu::VertBuf::new("overlay.bbox.instances".to_string()),
            last: None,
        }
    }

    /// Rebuilds the 12-edge instance buffer from the document's bounds, only
    /// when they've actually changed.
    pub(crate) fn prepare(&mut self, ctx: &gpu::Context, aabb: &Aabb3) {
        if self.last == Some(*aabb) {
            return;
        }
        self.last = Some(*aabb);
        if aabb.is_empty() {
            return;
        }
        let corners = aabb.corners();
        let data: Vec<EdgeInstance> = Aabb3::EDGES
            .iter()
            .map(|&(a, b)| EdgeInstance {
                v0: corners[a as usize].into(),
                v1: corners[b as usize].into(),
            })
            .collect();
        self.instances.update(ctx, &data);
    }

    pub(crate) fn draw(&self, ctx: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        if self.last.is_none_or(|aabb| aabb.is_empty()) {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, ctx.shared.buffers.rect.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.slice());
        render_pass.draw(0..4, 0..12);
    }
}
