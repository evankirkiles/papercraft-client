use crate::{camera, texture, Renderable};

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct Viewport3d {
    camera: camera::Camera,
    render_pipeline: wgpu::RenderPipeline,
    dims: Rect,
}

impl Viewport3d {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        dims: Rect,
    ) -> Self {
        let camera = camera::Camera::new(device, dims.w / dims.h);

        let shader = device
            .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Pipeline Layout"),
                bind_group_layouts: &[], // TODO: These Bind groups
                push_constant_ranges: &[],
            });
        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: config.format,
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
                depth_stencil: None,
                // depth_stencil: Some(wgpu::DepthStencilState {
                //     format: texture::Texture::DEPTH_FORMAT,
                //     depth_write_enabled: true,
                //     depth_compare: wgpu::CompareFunction::Less,
                //     stencil: wgpu::StencilState::default(),
                //     bias: wgpu::DepthBiasState::default(),
                // }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            camera,
            render_pipeline,
            dims,
        }
    }
}

impl Renderable for Viewport3d {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_viewport(
            self.dims.x,
            self.dims.y,
            self.dims.w,
            self.dims.h,
            0.0,
            1.0,
        );
        // render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}
