use std::mem;

use crate::gpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera2DUniform {
    view_proj: [[f32; 4]; 4],
    dimensions: [f32; 2],
    // Extra padding bits to bring up to "144" size
    padding: [f32; 2],
}

impl Camera2DUniform {
    fn new(camera: &pp_core::viewport::d2::camera::Camera2D, width: f32, height: f32) -> Self {
        let aspect = width.max(1.0) / height.max(1.0);
        let half_width = aspect / camera.zoom;
        let half_height = 1.0 / camera.zoom;
        let view = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            -1.0 * camera.eye.x,
            -1.0 * camera.eye.y,
            0.0,
        ));
        let proj = cgmath::ortho(-half_width, half_width, -half_height, half_height, -1.0, 1.0);
        let view_proj = proj * view;
        Self { dimensions: [width, height], view_proj: view_proj.into(), padding: [0.0, 0.0] }
    }
}

#[derive(Debug)]
pub struct Camera2DGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,

    /// Dimensions of the camera, used for consistent pixel size when necessary
    width: f32,
    height: f32,
}

impl Camera2DGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "viewport_2d.camera".to_string(),
            mem::size_of::<Camera2DUniform>(),
        );
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport_2d.camera"),
            layout: &ctx.shared_layouts.bind_groups.camera,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
        });
        Self { buf, bind_group, width: 1.0, height: 1.0 }
    }

    pub fn sync(
        &mut self,
        ctx: &gpu::Context,
        camera: &pp_core::viewport::d2::camera::Camera2D,
        width: f32,
        height: f32,
    ) {
        if self.width != width || self.height != height || camera.is_dirty {
            self.buf.update(ctx, &[Camera2DUniform::new(camera, width, height)]);
        }
        self.width = width;
        self.height = height;
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }
}