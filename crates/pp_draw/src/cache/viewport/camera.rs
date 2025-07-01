use pp_editor::{measures::Dimensions, viewport::camera::Camera};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    dimensions: [f32; 2],
    padding: [f32; 2], // Extra padding bits to bring up to correct alignment
}

impl CameraUniform {
    pub fn new(camera: &impl Camera, dims: Dimensions<f32>) -> Self {
        Self {
            view_proj: camera.view_proj(dims).into(),
            eye: camera.eye(),
            dimensions: dims.into(),
            padding: [0.0, 0.0],
        }
    }
}
