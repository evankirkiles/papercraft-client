use pp_core::MaterialId;

use crate::cache;
use crate::gpu;

use super::DepthBiasLayer;

/// `SurfaceProgram` renders possibly textured faces.
#[derive(Debug)]
pub struct SurfaceProgram {
    pipeline: wgpu::RenderPipeline,
}

impl SurfaceProgram {
    pub(super) fn new(ctx: &gpu::Context) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("./shaders/surface.wgsl"));
        Self {
            pipeline: ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ink3.surface"),
                layout: Some(&ctx.shared.pipeline_layouts.mesh_surface),
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
                        format: ctx.view_format,
                        blend: Some(wgpu::BlendState::REPLACE),
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
                    bias: wgpu::DepthBiasState {
                        constant: DepthBiasLayer::BackgroundTop as i32,
                        slope_scale: 0.05,
                        ..Default::default()
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: (&ctx.settings.msaa_level).into(),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
        }
    }

    // Writes geometry draw commands for all the materials in a mesh
    // pub(super) fn draw_mesh(
    //     &self,
    //     ctx: &gpu::Context,
    //     render_pass: &mut wgpu::RenderPass,
    //     mesh: &cache::MeshGPU,
    // ) {
    //     render_pass.set_pipeline(&self.pipeline);
    //     mesh.draw_tris(ctx, render_pass);
    // }

    pub(super) fn draw_mesh_with_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &MaterialId,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_material_surface(ctx, render_pass, material_id);
    }

    /// Writes geometry draw commands for all the materials in a mesh
    pub fn draw_piece_mesh_with_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &MaterialId,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        mesh.draw_piece_material_surface(ctx, render_pass, material_id);
    }
}
