use std::{collections::HashMap, mem, ops::Range};

use cgmath::SquareMatrix;
use pp_core::id;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

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
        Self { affine: piece.transform.into() }
    }
}

/// Pieces maintain their own affine transformation matrix uniform buffers, so
/// we can translate / rotate all of the faces within a piece easily.
#[derive(Debug)]
pub(crate) struct PieceGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,

    /// The range of elements in this piece in piecewise VBOs
    pub range: Range<u32>,
    /// The range of elements in this piece in material-piecewise VBOs
    /// A missing entry indicates that no face in this piece uses the given material.
    pub mat_ranges: HashMap<id::MaterialId, Range<u32>>,
}

impl PieceGPU {
    pub fn new(ctx: &gpu::Context, label: &str) -> Self {
        let buf = gpu::UniformBuf::new(ctx, label.to_string(), mem::size_of::<PieceUniform>());
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &ctx.shared.bind_group_layouts.piece,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            range: 0..0,
            mat_ranges: HashMap::new(),
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
                layout: &ctx.shared.bind_group_layouts.piece,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            range: 0..0,
            mat_ranges: HashMap::new(),
        }
    }

    pub fn sync(&mut self, ctx: &gpu::Context, piece: &pp_core::mesh::piece::Piece) {
        if piece.elem_dirty {
            self.buf.update(ctx, &[PieceUniform::new(piece)])
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Piece.value(), &self.bind_group, &[]);
    }
}
