use crate::gpu;
use pp_core::id;
use pp_core::mesh::{Mesh, MeshElementType};
use pp_core::select::SelectionState;
use std::collections::HashMap;

mod extract;

/// All the possible VBOs a mesh might need to use.
#[derive(Debug)]
pub struct MeshGPUVBOs {
    // For vertices & faces
    pub pos: gpu::VertBuf,
    pub nor: gpu::VertBuf,
    pub uv: gpu::VertBuf,
    pub pos_2d: gpu::VertBuf,
    // Edit flags (select state, active state, etc)
    pub vert_idx: gpu::VertBuf,
    pub vert_flags: gpu::VertBuf,
    pub face_idx: gpu::VertBuf,
    pub face_flags: gpu::VertBuf,

    // For edges
    pub edge_pos: gpu::VertBuf,
    pub edge_idx: gpu::VertBuf,
    pub edge_flags: gpu::VertBuf,
    // For cut edges
    pub cut_edge_pos: gpu::VertBuf,
}

/// All the IBOs which a mesh might need to use.
#[derive(Debug)]
pub struct MeshGPUIBOs {
    pub tris: gpu::IndexBuf,
    pub lines: gpu::IndexBuf,
    pub points: gpu::IndexBuf,

    /// IBOs per material, for material-specific draw calls
    pub tris_per_mat: HashMap<id::MaterialId, gpu::IndexBuf>,
}

/// A manager for VBOs / IBOs derived from a mesh.
#[derive(Debug)]
pub struct MeshGPU {
    vbo: MeshGPUVBOs,
    ibo: MeshGPUIBOs,

    /// Forces updating of all GPU-side resources
    is_dirty: bool,
}

impl MeshGPU {
    /// Creates the GPU representation of a mesh and populates its buffers
    pub fn new(mesh: &Mesh) -> Self {
        let mesh_lbl = mesh.label.as_str();
        Self {
            is_dirty: true,
            vbo: MeshGPUVBOs {
                pos: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.pos")),
                nor: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.nor")),
                uv: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.uv")),
                pos_2d: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.pos_2d")),
                vert_flags: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.vert_flags")),
                vert_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.vert_idx")),
                cut_edge_pos: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.cut_edge_pos")),
                edge_pos: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.edge_pos")),
                edge_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.edge_idx")),
                edge_flags: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.edge_flags")),
                face_idx: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.face_idx")),
                face_flags: gpu::VertBuf::new(format!("{mesh_lbl}.vbo.face_flags")),
            },
            ibo: MeshGPUIBOs {
                tris: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.tris")),
                lines: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.lines")),
                points: gpu::IndexBuf::new(format!("{mesh_lbl}.ibo.points")),
                tris_per_mat: HashMap::new(),
            },
        }
    }

    /// Updates any outdated VBOs / IBOs in this mesh
    pub fn sync(&mut self, ctx: &gpu::Context, mesh: &mut Mesh, selection: &SelectionState) {
        let elem_dirty = if self.is_dirty { &MeshElementType::all() } else { &mesh.elem_dirty };
        let index_dirty = if self.is_dirty { &MeshElementType::all() } else { &mesh.index_dirty };
        if elem_dirty.intersects(MeshElementType::VERTS) {
            extract::vbo::pos(ctx, mesh, &mut self.vbo.pos);
            extract::vbo::vnor(ctx, mesh, &mut self.vbo.nor);
            extract::vbo::edge_pos(ctx, mesh, &mut self.vbo.edge_pos);
            extract::vbo::cut_edge_pos(ctx, mesh, &mut self.vbo.cut_edge_pos);
        }
        if elem_dirty.intersects(MeshElementType::EDGES) {
            extract::vbo::cut_edge_pos(ctx, mesh, &mut self.vbo.cut_edge_pos);
        }
        if index_dirty.intersects(MeshElementType::VERTS) {
            extract::vbo::vert_idx(ctx, mesh, &mut self.vbo.vert_idx);
        }
        if index_dirty.intersects(MeshElementType::EDGES) {
            extract::vbo::edge_idx(ctx, mesh, &mut self.vbo.edge_idx);
        }
        if index_dirty.intersects(MeshElementType::LOOPS) {
            extract::ibo::tris(ctx, mesh, &mut self.ibo.tris);
        }
        if self.is_dirty || selection.is_dirty {
            extract::vbo::vert_flags(ctx, mesh, selection, &mut self.vbo.vert_flags);
            extract::vbo::edge_flags(ctx, mesh, selection, &mut self.vbo.edge_flags);
        }
        mesh.elem_dirty = MeshElementType::empty();
        mesh.index_dirty = MeshElementType::empty();
        self.is_dirty = false;
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
    // For vertices & faces
    vertex_format!(pos Float32x3);
    vertex_format!(pos_2d Float32x2);
    vertex_format!(nor Float32x3);
    vertex_format!(uv Float32x2);
    vertex_format!(vert_flags Uint32);
    vertex_format!(vert_idx Uint32x2);
    vertex_format!(face_idx Float32);

    // For edges
    vertex_format!(edge_pos Float32x3);
    vertex_format!(edge_pos_2d Float32x3);
    vertex_format!(edge_flags Uint32);
    vertex_format!(edge_idx Float32);
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
    make_batch_impl!(tris surface { 0 => pos, 1 => nor, 2 => vert_flags });
    make_batch_impl!(tris surface_2d { 0 => pos_2d });
    make_batch_impl!(tris edit_triangles { 0 => pos });

    pub const BATCH_BUFFER_LAYOUT_EDIT_CUT_LINES_INSTANCED: &[wgpu::VertexBufferLayout<'static>] =
        &[
            wgpu::VertexBufferLayout {
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size() * 2,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS,
                        offset: 0,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS,
                        offset: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size(),
                        shader_location: 2,
                    },
                ],
            },
        ];

    pub const BATCH_BUFFER_LAYOUT_EDIT_LINES_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size() * 2,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS,
                    offset: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size(),
                    shader_location: 2,
                },
            ],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_FLAGS,
                offset: 0,
                shader_location: 3,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_IDX,
                offset: 0,
                shader_location: 4,
            }],
        },
    ];

    pub const BATCH_BUFFER_LAYOUT_EDIT_POINTS_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS,
                offset: 0,
                shader_location: 1,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS,
                offset: 0,
                shader_location: 2,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_IDX,
                offset: 0,
                shader_location: 3,
            }],
        },
    ];

    pub fn draw_edit_points_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo.pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo.vert_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo.vert_idx.slice());
        render_pass.draw(0..4, 0..self.vbo.vert_idx.len);
    }

    pub fn draw_edit_lines_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo.edge_pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo.edge_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo.edge_idx.slice());
        render_pass.draw(0..4, 0..self.vbo.edge_idx.len);
    }

    pub fn draw_edit_cut_lines_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.vbo.cut_edge_pos.len == 0 {
            return;
        }
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo.cut_edge_pos.slice());
        render_pass.draw(0..4, 0..self.vbo.cut_edge_pos.len);
    }
}
