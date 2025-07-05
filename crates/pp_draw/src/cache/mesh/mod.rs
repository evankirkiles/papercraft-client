use crate::gpu;
use extract::{ibo, vbo};
use piece::PieceGPU;
use pp_core::id::{self, Id};
use pp_core::mesh::{MaterialSlotId, Mesh, MeshElementType};
use pp_core::select::SelectionState;
use pp_core::{MaterialId, MeshId};
use slotmap::SecondaryMap;
use std::collections::HashMap;
use std::ops::Range;

mod extract;
pub mod piece;

/// Pieces maintain their own affine transformation matrix uniform buffers, so
/// we can translate / rotate all of the faces within a piece easily.
#[derive(Clone, Debug, Default)]
pub(crate) struct MaterialGPUVBORange {
    /// This material's range of elements in in the IBO for standard VBOs
    pub range: Range<u32>,
    /// This material's range of elements in the IBO for piecewise VBOs
    /// A missing entry indicates that the piece doesn't use the given material.
    pub piece_ranges: HashMap<id::PieceId, Range<u32>>,
}

/// All the possible VBOs a mesh might need to use.
#[derive(Debug)]
pub struct MeshGPUVBOs {
    // For vertices
    pub pos: gpu::VertBuf,
    pub nor: gpu::VertBuf,
    pub uv: gpu::VertBuf,
    pub vert_idx: gpu::VertBuf,
    pub vert_flags: gpu::VertBuf,

    // For edges (for which we use wholly separate VBOs to do instanced line rendering)
    pub edge_pos: gpu::VertBuf,
    pub edge_idx: gpu::VertBuf,
    pub edge_flags: gpu::VertBuf,
    pub edge_flap: gpu::VertBuf,

    // For per-material indexing. Each `MaterialSlotGPU` has a corresponding range in
    // this buffer to render all of the faces with that slot's material
    pub mat_indices: gpu::IndexBuf,
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
            edge_flap: gpu::VertBuf::new(format!("{label}.edge_flap")),
            mat_indices: gpu::IndexBuf::new(format!("{label}.mat_indices")),
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

    // Likewise, material slots own their ranges in the material index buffers
    mat_ranges: SecondaryMap<MaterialId, MaterialGPUVBORange>,

    /// Forces updating of *all* GPU-side resources
    is_dirty: bool,
}

impl MeshGPU {
    /// Creates the GPU representation of a mesh and populates its buffers
    pub fn new(mesh: &Mesh) -> Self {
        let mesh_lbl = mesh.label.as_str();
        Self {
            is_dirty: true,
            vbo: MeshGPUVBOs::new(&format!("{mesh_lbl}.vbo)")),
            vbo_pieces: MeshGPUVBOs::new(&format!("{mesh_lbl}.vbo_pieces)")),
            pieces: HashMap::new(),
            mat_ranges: SecondaryMap::new(),
        }
    }

    /// Updates any outdated VBOs / IBOs in this mesh
    pub fn sync(
        &mut self,
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &mut Mesh,
        slots: &SecondaryMap<MaterialSlotId, MaterialId>,
        default_mat: &MaterialId,
        selection: &SelectionState,
    ) {
        let elem_dirty = if self.is_dirty { &MeshElementType::all() } else { &mesh.elem_dirty };
        let index_dirty = if self.is_dirty { &MeshElementType::all() } else { &mesh.index_dirty };
        if elem_dirty.intersects(MeshElementType::VERTS) {
            // Meshwise VBOs
            vbo::pos(ctx, mesh, &mut self.vbo.pos);
            vbo::uv(ctx, mesh, &mut self.vbo.uv);
            vbo::vnor(ctx, mesh, &mut self.vbo.nor);
            vbo::edge_pos(ctx, mesh, &mut self.vbo.edge_pos);
            // Piecewise VBOs
            vbo::piece_pos(ctx, mesh, &mut self.vbo_pieces.pos);
            vbo::piece_uv(ctx, mesh, &mut self.vbo_pieces.uv);
            vbo::piece_vnor(ctx, mesh, &mut self.vbo_pieces.nor);
            vbo::piece_edge_pos(ctx, mesh, &mut self.vbo_pieces.edge_pos);
            vbo::piece_edge_flap(ctx, mesh, &mut self.vbo_pieces.edge_flap);
            // Material slot IBOs
            let ranges = &mut self.mat_ranges;
            ibo::mat_indices(ctx, mesh, slots, default_mat, &mut self.vbo.mat_indices, ranges);
            ibo::piece_mat_indices(
                ctx,
                mesh,
                slots,
                default_mat,
                &mut self.vbo_pieces.mat_indices,
                ranges,
            );
        }
        if elem_dirty.intersects(MeshElementType::EDGES) {
            vbo::edge_flags(ctx, m_id, mesh, selection, &mut self.vbo.edge_flags);
        }
        if elem_dirty.intersects(MeshElementType::FLAPS) {
            vbo::piece_edge_flap(ctx, mesh, &mut self.vbo_pieces.edge_flap);
        }
        if index_dirty.intersects(MeshElementType::VERTS) {
            vbo::vert_idx(ctx, m_id, mesh, &mut self.vbo.vert_idx);
        }
        if index_dirty.intersects(MeshElementType::EDGES) {
            vbo::edge_idx(ctx, m_id, mesh, &mut self.vbo.edge_idx);
            vbo::piece_edge_idx(ctx, m_id, mesh, &mut self.vbo_pieces.edge_idx);
        }

        // TODO: Clean this up
        if index_dirty.intersects(MeshElementType::PIECES) {
            vbo::vert_idx(ctx, m_id, mesh, &mut self.vbo.vert_idx);
            vbo::piece_pos(ctx, mesh, &mut self.vbo_pieces.pos);
            vbo::piece_uv(ctx, mesh, &mut self.vbo_pieces.uv);
            vbo::piece_vnor(ctx, mesh, &mut self.vbo_pieces.nor);
            vbo::piece_vert_idx(ctx, m_id, mesh, &mut self.vbo_pieces.vert_idx);
            vbo::piece_vert_flags(ctx, m_id, mesh, selection, &mut self.vbo_pieces.vert_flags);
            vbo::piece_edge_pos(ctx, mesh, &mut self.vbo_pieces.edge_pos);
            vbo::piece_edge_idx(ctx, m_id, mesh, &mut self.vbo_pieces.edge_idx);
            vbo::piece_edge_flap(ctx, mesh, &mut self.vbo_pieces.edge_flap);
            vbo::piece_edge_flags(ctx, m_id, mesh, selection, &mut self.vbo_pieces.edge_flags);
            // Material slot IBOs
            let ranges = &mut self.mat_ranges;
            ibo::piece_mat_indices(
                ctx,
                mesh,
                slots,
                default_mat,
                &mut self.vbo_pieces.mat_indices,
                ranges,
            );
            // Ensure each piece has up-to-date ranges of elements to render
            let mut i = 0;
            mesh.pieces.indices().for_each(|p_id| {
                let p_id = id::PieceId::from_usize(p_id);
                let piece = self
                    .pieces
                    .entry(p_id)
                    .or_insert_with(|| PieceGPU::new(ctx, format!("{p_id:?}").as_str()));
                let n_els = mesh
                    .iter_connected_faces(mesh[p_id].f)
                    .flat_map(|f_id| mesh.iter_face_loops(f_id))
                    .count() as u32;
                piece.range = i..(i + n_els);
                i += n_els;
            });
            // Delete pieces no longer being used
            if mesh.pieces.num_elements() != self.pieces.len() {
                let old_keys: Vec<_> = self
                    .pieces
                    .keys()
                    .filter_map(|p_id| {
                        if !mesh.pieces.has_element_at(p_id.to_usize()) {
                            Some(*p_id)
                        } else {
                            None
                        }
                    })
                    .collect();
                old_keys.iter().for_each(|p_id| {
                    self.pieces.remove_entry(p_id);
                });
            }
        }

        // If piece transforms have changed, make sure we sync all of them
        if elem_dirty.intersects(MeshElementType::PIECES) {
            mesh.pieces.iter_mut().for_each(|(p_id, piece)| {
                if piece.elem_dirty {
                    let p_id = id::PieceId::from_usize(p_id);
                    if let Some(piece_gpu) = self.pieces.get_mut(&p_id) {
                        piece_gpu.sync(ctx, piece)
                    }
                    piece.elem_dirty = false;
                }
            })
        }

        if self.is_dirty || selection.is_dirty {
            vbo::vert_flags(ctx, m_id, mesh, selection, &mut self.vbo.vert_flags);
            vbo::edge_flags(ctx, m_id, mesh, selection, &mut self.vbo.edge_flags);
            vbo::piece_vert_flags(ctx, m_id, mesh, selection, &mut self.vbo_pieces.vert_flags);
            vbo::piece_edge_flags(ctx, m_id, mesh, selection, &mut self.vbo_pieces.edge_flags);
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
    vertex_format!(uv Float32x2);
    vertex_format!(vert_flags Uint32);
    vertex_format!(vert_idx Uint32x4); // Face / Vert / Mesh(x2) idx

    // For edges
    vertex_format!(edge_pos Float32x3);
    vertex_format!(edge_flags Uint32);
    vertex_format!(edge_idx Uint32x4); // _ / Edge / Mesh(x2) idx

    // For flaps
    vertex_format!(flap_flags Uint32);
}

impl MeshGPU {
    pub const BATCH_BUFFER_LAYOUT_TRIS: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_POS.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_NOR.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_NOR,
                offset: 0,
                shader_location: 1,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS,
                offset: 0,
                shader_location: 2,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_VERT_IDX.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_IDX,
                offset: 0,
                shader_location: 3,
            }],
        },
    ];

    pub fn draw_tris(&self, _: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vbo.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo.vert_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo.vert_idx.slice());
        render_pass.draw(0..self.vbo.vert_idx.len, 0..1);
    }

    pub fn draw_piece_tris(&self, _: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        if self.pieces.is_empty() {
            return;
        };
        render_pass.set_vertex_buffer(0, self.vbo_pieces.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo_pieces.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.vert_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo_pieces.vert_idx.slice());
        // Draw ranges from the buffers by binding each uniform
        self.pieces.values().for_each(|piece| {
            piece.bind(render_pass);
            render_pass.draw(piece.range.clone(), 0..1);
        })
    }

    pub const BATCH_BUFFER_LAYOUT_EDIT_POINTS_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: wgpu::VertexFormat::Float32x2.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_POS.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS,
                offset: 0,
                shader_location: 1,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_VERT_FLAGS,
                offset: 0,
                shader_location: 2,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_VERT_IDX.size(),
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

    pub fn draw_piece_edit_points_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.pieces.is_empty() {
            return;
        };
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo_pieces.pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.vert_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo_pieces.vert_idx.slice());
        self.pieces.values().for_each(|piece| {
            piece.bind(render_pass);
            render_pass.draw(0..4, piece.range.clone());
        })
    }

    pub const BATCH_BUFFER_LAYOUT_EDIT_LINES_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: wgpu::VertexFormat::Float32x2.size(),
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
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_FLAGS.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_FLAGS,
                offset: 0,
                shader_location: 3,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_IDX.size(),
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

    pub fn draw_piece_edit_lines_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.pieces.is_empty() {
            return;
        };
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo_pieces.edge_pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.edge_flags.slice());
        render_pass.set_vertex_buffer(3, self.vbo_pieces.edge_idx.slice());
        self.pieces.values().for_each(|piece| {
            piece.bind(render_pass);
            render_pass.draw(0..4, piece.range.clone());
        })
    }

    pub const BATCH_BUFFER_LAYOUT_FLAPS_INSTANCED: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: wgpu::VertexFormat::Float32x2.size(),
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
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size()
                + MeshGPUVBOs::VERTEX_FORMAT_FLAP_FLAGS.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS,
                    offset: 0,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: MeshGPUVBOs::VERTEX_FORMAT_FLAP_FLAGS,
                    offset: MeshGPUVBOs::VERTEX_FORMAT_EDGE_POS.size(),
                    shader_location: 4,
                },
            ],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_FLAGS.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_FLAGS,
                offset: 0,
                shader_location: 5,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_EDGE_IDX.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_EDGE_IDX,
                offset: 0,
                shader_location: 6,
            }],
        },
    ];

    pub fn draw_piece_flaps_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.pieces.is_empty() {
            return;
        };
        render_pass.set_vertex_buffer(0, ctx.buf_rect.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo_pieces.edge_pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.edge_flap.slice());
        render_pass.set_vertex_buffer(3, self.vbo_pieces.edge_flags.slice());
        render_pass.set_vertex_buffer(4, self.vbo_pieces.edge_idx.slice());
        self.pieces.values().for_each(|piece| {
            piece.bind(render_pass);
            render_pass.draw(0..4, piece.range.clone());
        })
    }

    pub fn draw_piece_flaps_outline_instanced(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.pieces.is_empty() {
            return;
        };
        render_pass.set_vertex_buffer(0, ctx.buf_rect_outline.slice(..));
        render_pass.set_vertex_buffer(1, self.vbo_pieces.edge_pos.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.edge_flap.slice());
        render_pass.set_vertex_buffer(3, self.vbo_pieces.edge_flags.slice());
        render_pass.set_vertex_buffer(4, self.vbo_pieces.edge_idx.slice());
        self.pieces.values().for_each(|piece| {
            piece.bind(render_pass);
            render_pass.draw(0..24, piece.range.clone());
        })
    }

    pub const BATCH_BUFFER_LAYOUT_SURFACE: &[wgpu::VertexBufferLayout<'static>] = &[
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_POS.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_POS,
                offset: 0,
                shader_location: 0,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_NOR.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_NOR,
                offset: 0,
                shader_location: 1,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: MeshGPUVBOs::VERTEX_FORMAT_UV.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: MeshGPUVBOs::VERTEX_FORMAT_UV,
                offset: 0,
                shader_location: 2,
            }],
        },
    ];

    pub fn draw_surface(&self, _: &gpu::Context, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vbo.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo.uv.slice());
        render_pass.draw(0..self.vbo.vert_idx.len, 0..1);
    }

    pub fn draw_material_surface(
        &self,
        _: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        material_id: &MaterialId,
    ) {
        let Some(material) = self.mat_ranges.get(*material_id) else {
            return;
        };
        render_pass.set_index_buffer(self.vbo.mat_indices.slice(), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vbo.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo.uv.slice());
        render_pass.draw_indexed(material.range.clone(), 0, 0..1);
    }

    pub fn draw_piece_material_surface(
        &self,
        _: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        material_id: &MaterialId,
    ) {
        if self.pieces.is_empty() {
            return;
        };
        let Some(material) = self.mat_ranges.get(*material_id) else {
            return;
        };
        render_pass
            .set_index_buffer(self.vbo_pieces.mat_indices.slice(), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vbo_pieces.pos.slice());
        render_pass.set_vertex_buffer(1, self.vbo_pieces.nor.slice());
        render_pass.set_vertex_buffer(2, self.vbo_pieces.uv.slice());
        // The material is bound, so draw all the pieces which use it
        material.piece_ranges.iter().for_each(|(p_id, range)| {
            self.pieces[p_id].bind(render_pass);
            render_pass.draw_indexed(range.clone(), 0, 0..1);
        })
    }
}
