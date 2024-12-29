use crate::gpu;
use std::collections::HashMap;

use super::buffers::MeshBuffers;

pub struct MeshBufferBatches {
    // For "inking", e.g. textured surfaces (3D)
    pub surface: gpu::Batch,
    pub surface_per_mat: HashMap<u32, gpu::Batch>,
    // For "inking", e.g. textured surfaces (2D)
    pub surface_2d: gpu::Batch,
    pub surface_2d_per_mat: HashMap<u32, gpu::Batch>,

    // For "overlay", e.g. wireframes / editing primitives
    pub edit_triangles: gpu::Batch,
    pub edit_vertices: gpu::Batch,
    pub edit_edges: gpu::Batch,
}

pub mod batch_buffer_layouts {
    use wgpu::vertex_attr_array;

    /// Creates an array of single-attribute VBO Layouts for use in shader programs
    /// These *must* match the order /.actual buffers created in `new` below.
    macro_rules! make_batch_buffer_layouts {
        ( $(#[$attr:meta])* $name:ident = {$($loc:expr => $fmt:ident),* $(,)?}; ) => {
            pub const $name: &[wgpu::VertexBufferLayout] = &[
            $(wgpu::VertexBufferLayout {
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attr_array![ $loc => $fmt ],
            }),+];
        }
    }

    make_batch_buffer_layouts! { SURFACE = { 0 => Float32x3, 1 => Float32x3 }; } // pos, nor
    make_batch_buffer_layouts! { SURFACE_2D = { 0 => Float32x2 }; }
    make_batch_buffer_layouts! { EDIT_TRIANGLES = { 0 => Float32x3 }; }
    make_batch_buffer_layouts! { EDIT_VERTICES = { 0 => Float32x3 }; }
    make_batch_buffer_layouts! { EDIT_EDGES = { 0 => Float32x3 }; }
}

impl MeshBufferBatches {
    pub fn new(bufs: &MeshBuffers) -> Self {
        Self {
            surface: gpu::Batch {
                vbos: vec![
                    bufs.vbo.pos.clone(),
                    bufs.vbo.nor.clone(), /* bufs.vbo.uv.clone() */
                ],
                ibo: bufs.ibo.tris.clone(),
            },
            surface_2d: gpu::Batch {
                vbos: vec![bufs.vbo.pos_2d.clone()],
                ibo: bufs.ibo.tris.clone(),
            },
            edit_triangles: gpu::Batch {
                vbos: vec![bufs.vbo.pos.clone()],
                ibo: bufs.ibo.tris.clone(),
            },
            edit_vertices: gpu::Batch {
                vbos: vec![bufs.vbo.pos.clone()],
                ibo: bufs.ibo.points.clone(),
            },
            edit_edges: gpu::Batch {
                vbos: vec![bufs.vbo.pos.clone()],
                ibo: bufs.ibo.lines.clone(),
            },
            // Lookup tables for per-material batches
            surface_per_mat: HashMap::new(),
            surface_2d_per_mat: HashMap::new(),
        }
    }
}
