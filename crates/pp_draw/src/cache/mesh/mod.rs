use crate::gpu;
use piece::PieceGPU;
use pp_core::id;
use pp_core::mesh::{Mesh, MeshElementType};
use pp_core::select::SelectionState;
use std::collections::HashMap;

mod extract;
mod piece;

/// All the possible VBOs a mesh might need to use.
#[derive(Debug)]
pub struct MeshGPUVBOs {
    // For vertices
    pub pos: gpu::VertBuf,
    pub nor: gpu::VertBuf,
    pub uv: gpu::VertBuf,
    pub vert_idx: gpu::VertBuf,
    pub vert_flags: gpu::VertBuf,

    // For edges
    pub edge_pos: gpu::VertBuf,
    pub edge_idx: gpu::VertBuf,
    pub edge_flags: gpu::VertBuf,

    // For faces
    pub face_idx: gpu::VertBuf,
    pub face_flags: gpu::VertBuf,

    // For cut edges
    pub cut_edge_pos: gpu::VertBuf,
}

impl MeshGPUVBOs {
    fn new(label: &str) -> Self {
        Self {
            pos: gpu::VertBuf::new(format!("{label}.pos")),
            nor: gpu::VertBuf::new(format!("{label}.nor")),
            uv: gpu::VertBuf::new(format!("{label}.uv")),
            vert_flags: gpu::VertBuf::new(format!("{label}.vert_flags")),
            vert_idx: gpu::VertBuf::new(format!("{label}.vert_idx")),
            edge_pos: gpu::VertBuf::new(format!("{label}.edge_pos")),
            edge_idx: gpu::VertBuf::new(format!("{label}.edge_idx")),
            edge_flags: gpu::VertBuf::new(format!("{label}.edge_flags")),
            face_idx: gpu::VertBuf::new(format!("{label}.face_idx")),
            face_flags: gpu::VertBuf::new(format!("{label}.face_flags")),
            cut_edge_pos: gpu::VertBuf::new(format!("{label}.cut_edge_pos")),
        }
    }
}

/// A manager for VBOs / IBOs derived from a mesh.
#[derive(Debug)]
pub struct MeshGPU {
    /// VBOs with vertices laid out in loop order. Used for the "reference" mesh
    /// before pieces have been cut out, as we don't need to worry about binding
    /// uniform buffers for piece transforms.
    vbo: MeshGPUVBOs,

    /// VBOs with vertices laid out in piece -> loop order. These VBOs have the
    /// same capacity as the loop-only VBO, but have contents that are progressively
    /// filled as valid pieces are created. Drawing a piece and all of its edit
    /// components then becomes drawing the proper range of vertices in these VBOs.
    vbo_pieces: MeshGPUVBOs,

    // Pieces own their own uniform buffers and the ranges of their vertices
    // in the piece VBOs.
    pieces: HashMap<id::PieceId, PieceGPU>,

    /// Forces updating of *all* GPU-side resources
    is_dirty: bool,
}

impl MeshGPU {
    /// Creates the GPU representation of a mesh and populates its buffers
    pub fn new(mesh: &Mesh) -> Self {
        let mesh_lbl = mesh.label.as_str();
        Self {
            is_dirty: true,
            pieces: HashMap::new(),
            vbo: MeshGPUVBOs::new(&format!("{mesh_lbl}.vbo)")),
            vbo_pieces: MeshGPUVBOs::new(&format!("{mesh_lbl}.vbo_pieces)")),
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
            extract::vbo::piece_pos(ctx, mesh, &mut self.vbo_pieces.pos);
            extract::vbo::piece_vnor(ctx, mesh, &mut self.vbo_pieces.nor);
            extract::vbo::piece_vert_flags(ctx, mesh, selection, &mut self.vbo_pieces.vert_flags);
        }
        if index_dirty.intersects(MeshElementType::VERTS) {
            extract::vbo::vert_idx(ctx, mesh, &mut self.vbo.vert_idx);
        }
        if index_dirty.intersects(MeshElementType::EDGES) {
            extract::vbo::edge_idx(ctx, mesh, &mut self.vbo.edge_idx);
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
    // For vertices
    vertex_format!(pos Float32x3);
    vertex_format!(nor Float32x3);
    // vertex_format!(uv Float32x2);
    vertex_format!(vert_flags Uint32);
    vertex_format!(vert_idx Uint32x2);

    // For edges
    vertex_format!(edge_pos Float32x3);
    vertex_format!(edge_flags Uint32);
    vertex_format!(edge_idx Float32);

    // For faces
    // vertex_format!(face_idx Float32);
}

impl MeshGPU {
    pub const BATCH_BUFFER_LAYOUT_TRIS: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
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
                format: MeshGPUVBOs::VERTEX_FORMAT_NOR,
                offset: 0,
                shader_location: 1,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS,
                offset: 0,
                shader_location: 2,
            }],
        },
    ];

    pub fn draw_tris(&self, _: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vbo.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo.vert_flags.slice());
        render_pass.draw(0..self.vbo.vert_idx.len, 0..1);
    }

    pub fn draw_piece_tris(&self, _: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        if self.vbo_pieces.pos.len == 0 {
            return;
        }
        render_pass.set_vertex_buffer(0, self.vbo_pieces.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo_pieces.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.vert_flags.slice());
        render_pass.draw(0..self.vbo_pieces.pos.len, 0..1);
    }

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
