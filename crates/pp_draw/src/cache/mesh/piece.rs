use std::{mem, ops::Range};

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PieceUniform {
    affine: [[f32; 4]; 4],
    /// This piece's position in `[0, 1)` within its depth class, breaking ties
    /// between coplanar pieces. See `engines::ink::DepthClass`.
    depth_slot: f32,
    _pad: [f32; 3],
}

impl PieceUniform {
    fn new(piece: &pp_core::mesh::piece::Piece, depth_slot: f32) -> Self {
        Self { depth_slot, ..Self::from_matrix(piece.transform) }
    }

    fn from_matrix(m: cgmath::Matrix4<f32>) -> Self {
        Self { affine: m.into(), depth_slot: 0.0, _pad: [0.0; 3] }
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
    bind_group: wgpu::BindGroup,

    /// Last-synced depth slot, so a reordering of pieces re-uploads the uniform
    /// even when the transform itself hasn't moved.
    depth_slot: f32,

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
            depth_slot: f32::NAN,
            range: 0..0,
        }
    }

    /// Syncs this piece's uniform. `depth_slot` is the piece's position in
    /// `[0, 1)` among the mesh's pieces, which orders coplanar pieces against
    /// each other within a depth class.
    pub fn sync(
        &mut self,
        ctx: &gpu::Context,
        piece: &pp_core::mesh::piece::Piece,
        depth_slot: f32,
    ) {
        if piece.elem_dirty || self.depth_slot != depth_slot {
            self.depth_slot = depth_slot;
            self.buf.update(ctx, &[PieceUniform::new(piece, depth_slot)])
        }
    }

    /// Syncs this uniform buffer against a mesh's own model transform (used
    /// for the single per-mesh model uniform, not per-piece uniforms).
    pub fn sync_from_mesh(&mut self, ctx: &gpu::Context, mesh: &mut pp_core::mesh::Mesh) {
        if mesh.uniform_dirty {
            self.buf.update(ctx, &[PieceUniform::from_matrix(mesh.transform)]);
            mesh.uniform_dirty = false;
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Piece.value(), &self.bind_group, &[]);
    }
}
