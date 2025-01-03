use std::collections::HashMap;

use pp_core::{
    id,
    mesh::{Mesh, MeshElementType},
};

use super::prelude::*;
use crate::gpu;

mod extract;

/// All the possible VBOs a mesh might need to use.
pub struct MeshGPUVBOs {
    // 3D View
    pub pos: gpu::VertBuf,
    pub nor: gpu::VertBuf,
    pub uv: gpu::VertBuf,
    // 2D View
    pub pos_2d: gpu::VertBuf,
    // Selection Indices
    pub vert_idx: gpu::VertBuf,
    pub edge_idx: gpu::VertBuf,
    pub face_idx: gpu::VertBuf,

    // Rect (for points)
    pub edit_rect: gpu::VertBuf,
}

/// All the IBOs which a mesh might need to use.
pub struct MeshGPUIBOs {
    pub tris: gpu::IndexBuf,
    pub lines: gpu::IndexBuf,
    pub points: gpu::IndexBuf,

    /// IBOs per material, for material-specific draw calls
    pub tris_per_mat: HashMap<id::MaterialId, gpu::IndexBuf>,
}

/// A manager for VBOs / IBOs derived from a mesh.
pub struct MeshGPU {
    vbo: MeshGPUVBOs,
    ibo: MeshGPUIBOs,

    /// Forces updating of all GPU-side resources
    is_dirty: bool,
}

impl GPUCache<Mesh> for MeshGPU {
    /// Creates the GPU representation of a mesh and populates its buffers
    fn new(ctx: &gpu::Context, mesh: &Mesh) -> Self {
        // let batches = MeshBufferBatches::new(&bufs);
        let mesh_lbl = mesh.label.as_str();
        let mut cache_item = Self {
            is_dirty: true,
            vbo: MeshGPUVBOs {
                pos: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.pos")),
                nor: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.nor")),
                uv: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.uv")),
                pos_2d: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.pos_2d")),
                vert_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.vert_idx")),
                edge_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.edge_idx")),
                face_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.face_idx")),
                edit_rect: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.edit_rect")),
            },
            ibo: MeshGPUIBOs {
                tris: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.tris")),
                lines: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.lines")),
                points: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.points")),
                tris_per_mat: HashMap::new(),
            },
        };
        cache_item.sync(ctx, mesh);
        cache_item
    }

    /// Updates any outdated VBOs / IBOs in this mesh
    fn sync(&mut self, ctx: &gpu::Context, mesh: &Mesh) {
        let dirty_flags = if self.is_dirty { &MeshElementType::all() } else { &mesh.elem_dirty };
        if dirty_flags.intersects(MeshElementType::VERTS) {
            extract::vbo::pos(ctx, mesh, &mut self.vbo.pos);
            extract::vbo::vnor(ctx, mesh, &mut self.vbo.nor);
        }
        if dirty_flags.intersects(MeshElementType::LOOPS) {
            extract::ibo::tris(ctx, mesh, &mut self.ibo.tris);
        }
        if dirty_flags.intersects(MeshElementType::EDGES) {
            extract::ibo::lines(ctx, mesh, &mut self.ibo.lines);
        }
        self.is_dirty = false
    }
}

use paste::paste;

/// Creates a static wgpu::VertexFormat object to use for referencing
/// these attributes' formats. Helps in deduplication of batch layouts.
macro_rules! vertex_format {
    ($name:ident $format:ident) => {
        paste! {
            pub const [<VERTEX_FORMAT_ $name:upper>]: wgpu::VertexFormat = wgpu::VertexFormat::$format;
        }
    };
}

impl MeshGPUVBOs {
    vertex_format!(pos Float32x3);
    vertex_format!(nor Float32x3);
    vertex_format!(uv Float32x2);
    vertex_format!(pos_2d Float32x2);
    vertex_format!(vert_idx Float32);
    vertex_format!(edge_idx Float32);
    vertex_format!(face_idx Float32);
}

/// Creates shared VertexBufferLayouts to use in engine functions, as well as
/// the implementation of draw calls for each batch type.
macro_rules! make_batch_impl {
    ($ibo:ident $name:ident {$($loc:expr => $vbo:ident),* $(,)?}) => {
        paste! {
            pub const [<BATCH_BUFFER_LAYOUT_ $name:upper>]: &[wgpu::VertexBufferLayout<'static>] = &[
                $(wgpu::VertexBufferLayout {
                    array_stride: 0,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: MeshGPUVBOs::[<VERTEX_FORMAT_ $vbo:upper>],
                        offset: 0,
                        shader_location: $loc,
                    }],
                }),+
            ];

            pub fn [<draw_ $name>](&self, render_pass: &mut wgpu::RenderPass) {
                $(render_pass.set_vertex_buffer($loc, self.vbo.$vbo.slice());)+
                render_pass.set_index_buffer(self.ibo.$ibo.slice(), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.ibo.$ibo.len, 0, 0..1);
            }
        }
    };
}

impl MeshGPU {
    make_batch_impl!(tris surface { 0 => pos, 1 => nor});
    make_batch_impl!(tris surface_2d { 0 => pos_2d });
    make_batch_impl!(tris edit_triangles { 0 => pos });
    make_batch_impl!(lines edit_lines { 0 => pos });
    // make_batch_impl!(points edit_points { 0 => pos });

    pub const BATCH_BUFFER_LAYOUT_EDIT_POINTS_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS_2D,
                offset: 0,
                shader_location: 1,
            }],
        },
    ];

    pub fn draw_edit_points_instanced(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vbo.pos.slice());
        render_pass.draw(0..4, 0..self.vbo.pos.len);
    }
}
