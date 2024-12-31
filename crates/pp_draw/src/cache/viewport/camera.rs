use std::mem;

use crate::gpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(camera: &pp_core::viewport::camera::CameraPerspective3D, aspect: f32) -> Self {
        let view = cgmath::Matrix4::look_at_rh(camera.eye, camera.target, camera.up);
        let proj = cgmath::perspective(cgmath::Deg(camera.fovy), aspect, camera.znear, camera.zfar);
        Self { view_proj: (proj * view).into() }
    }
}

pub struct CameraGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,

    /// The aspect ratio of the Camera, derived from the viewport's dimensions
    aspect: f32,
}

impl CameraGPU {
    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(
            ctx,
            "viewport.camera".to_string(),
            mem::size_of::<CameraUniform>(),
        );
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport.camera"),
            layout: &ctx.shared_layouts.bind_groups.camera_3d,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
        });
        Self { buf, bind_group, aspect: 1.0 }
    }

    pub fn sync(
        &mut self,
        ctx: &gpu::Context,
        camera: &pp_core::viewport::camera::CameraPerspective3D,
        aspect: f32,
    ) {
        if self.aspect != aspect || camera.is_dirty {
            self.buf.update(ctx, &[CameraUniform::new(camera, aspect)]);
        }
        self.aspect = aspect
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }
}
