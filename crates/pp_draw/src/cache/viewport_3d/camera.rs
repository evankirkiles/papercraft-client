use crate::gpu::{self, layouts::bind_groups::UniformBindGroup};
use pp_core::viewport_3d::camera;
use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera3DUniform {
    view_proj: [[f32; 4]; 4],
    dimensions: [f32; 2],
    // Extra padding bits to bring up to "144" size
    padding: [f32; 2],
}

impl Camera3DUniform {
    fn new(camera: &camera::Camera3D, width: f32, height: f32) -> Self {
        let aspect = width.max(1.0) / height.max(1.0);
        let view = cgmath::Matrix4::look_at_rh(camera.eye, camera.target, camera.up);
        let proj = cgmath::perspective(cgmath::Deg(camera.fovy), aspect, camera.znear, camera.zfar);
        let view_proj = proj * view;
        Self { dimensions: [width, height], view_proj: view_proj.into(), padding: [0.0, 0.0] }
    }
}

#[derive(Debug)]
pub struct Camera3DGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,

    /// Dimensions of the camera, used for consistent pixel size when necessary
    width: f32,
    height: f32,
}

impl Camera3DGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "viewport.camera".to_string(),
            mem::size_of::<Camera3DUniform>(),
        );
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport.camera"),
            layout: &ctx.shared_layouts.bind_groups.camera,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
        });
        Self { buf, bind_group, width: 1.0, height: 1.0 }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, camera: &camera::Camera3D, width: f32, height: f32) {
        if self.width != width || self.height != height || camera.is_dirty {
            self.buf.update(ctx, &[Camera3DUniform::new(camera, width, height)]);
        }
        self.width = width;
        self.height = height;
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(UniformBindGroup::Camera.value(), &self.bind_group, &[]);
    }
}
