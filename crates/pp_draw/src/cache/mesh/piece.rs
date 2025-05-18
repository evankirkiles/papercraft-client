use crate::gpu;

#[derive(Debug)]
pub(crate) struct PieceGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
}

impl PieceGPU {}
