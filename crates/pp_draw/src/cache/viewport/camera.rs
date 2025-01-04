use std::mem;

use cgmath::SquareMatrix;

use crate::gpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_proj_inv: [[f32; 4]; 4],
    dimensions: [f32; 2],
    // Extra padding bits to bring up to "144" size
    padding: [f32; 2],
}

impl CameraUniform {
    fn new(
        camera: &pp_core::viewport::camera::CameraPerspective3D,
        width: f32,
        height: f32,
    ) -> Self {
        let aspect = width / height;
        let view = cgmath::Matrix4::look_at_rh(camera.eye, camera.target, camera.up);
        let proj = cgmath::perspective(cgmath::Deg(camera.fovy), aspect, camera.znear, camera.zfar);
        let view_proj = proj * view;
        Self {
            dimensions: [width, height],
            view_proj: view_proj.into(),
            view_proj_inv: view_proj.invert().unwrap().into(),
            padding: [0.0, 0.0],
        }
    }
}

pub struct CameraGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,

    /// Dimensions of the camera, used for consistent pixel size when necessary
    width: f32,
    height: f32,
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
        Self { buf, bind_group, width: 1.0, height: 1.0 }
    }

    pub fn sync(
        &mut self,
        ctx: &gpu::Context,
        camera: &pp_core::viewport::camera::CameraPerspective3D,
        width: f32,
        height: f32,
    ) {
        if self.width != width || self.height != height || camera.is_dirty {
            self.buf.update(ctx, &[CameraUniform::new(camera, width, height)]);
        }
        self.width = width;
        self.height = height;
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }
}
