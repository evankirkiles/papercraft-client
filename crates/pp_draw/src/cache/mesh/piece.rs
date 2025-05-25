use std::mem;

use cgmath::SquareMatrix;

use crate::gpu::{self, layouts::bind_groups::UniformBindGroup};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PieceUniform {
    affine: [[f32; 4]; 4],
}

impl Default for PieceUniform {
    fn default() -> Self {
        Self { affine: cgmath::Matrix4::identity().into() }
    }
}

impl PieceUniform {
    fn new(piece: &pp_core::mesh::piece::Piece) -> Self {
        let rotation_x = cgmath::Matrix4::from_angle_x(cgmath::Rad(piece.ro[0]));
        let rotation_y = cgmath::Matrix4::from_angle_y(cgmath::Rad(piece.ro[1]));
        let rotation_z = cgmath::Matrix4::from_angle_z(cgmath::Rad(piece.ro[2]));
        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::from(piece.po));
        let affine = translation * rotation_z * rotation_y * rotation_x;
        Self { affine: affine.into() }
    }
}

/// Pieces maintain their own affine transformation matrix uniform buffers, so
/// we can translate / rotate all of the faces within a piece easily.
#[derive(Debug)]
pub(crate) struct PieceGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
    /// The start of elements of this piece in piecewise VBOs
    pub i_start: u32,
    /// The end of elements of this piece in piecewise VBOs
    pub i_end: u32,
}

impl PieceGPU {
    pub fn new(ctx: &gpu::Context, label: &str) -> Self {
        let buf = gpu::UniformBuf::new(ctx, label.to_string(), mem::size_of::<PieceUniform>());
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &ctx.shared_layouts.bind_groups.piece,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            i_start: 0,
            i_end: 0,
        }
    }

    /// Creates an identity piece uniform, used to supply consistent uniform data
    /// for non-piece views without creating entirely separate pipelines.
    pub fn identity(ctx: &gpu::Context) -> Self {
        let label = "PieceGPU.identity".to_string();
        let buf = gpu::UniformBuf::init(ctx, label, &[PieceUniform::default()]);
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("PieceGPU.identity"),
                layout: &ctx.shared_layouts.bind_groups.piece,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            i_start: 0,
            i_end: 0,
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, piece: &pp_core::mesh::piece::Piece) {
        if piece.elem_dirty {
            self.buf.update(ctx, &[PieceUniform::new(piece)])
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(UniformBindGroup::Piece.value(), &self.bind_group, &[]);
    }
}
