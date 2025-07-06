use std::{mem, ops::Range};

use cgmath::SquareMatrix;

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

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

/// Pieces maintain their own affine transformation matrix uniform buffers, so
/// we can translate / rotate all of the faces within a piece easily.
#[derive(Debug)]
pub(crate) struct PieceGPU {
    buf: gpu::UniformBuf,
    pub bind_group: wgpu::BindGroup,

    /// The range of elements in this piece in non-material piecewise VBOs
    pub range: Range<u32>,
}

impl PieceGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("piece"),
            entries: &[PieceUniform::bind_group_layout_entry(0)],
        })
    }

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
        }
    }
}
