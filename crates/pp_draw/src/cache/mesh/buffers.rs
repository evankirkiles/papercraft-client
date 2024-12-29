use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pp_core::id::{self, Id};
use pp_core::mesh::MeshDirtyFlags;

use crate::gpu;

/// All the possible VBOs a mesh might need to use.
///
/// Wrap in Rc/RefCell to allow batches to point to these values.
pub struct MeshVBOs {
    // 3D View
    pub pos: Rc<RefCell<gpu::VertBuf>>,
    pub nor: Rc<RefCell<gpu::VertBuf>>,
    pub uv: Rc<RefCell<gpu::VertBuf>>,
    // 2D View
    pub pos_2d: Rc<RefCell<gpu::VertBuf>>,
    // Selection Indices
    pub vert_idx: Rc<RefCell<gpu::VertBuf>>,
    pub edge_idx: Rc<RefCell<gpu::VertBuf>>,
    pub face_idx: Rc<RefCell<gpu::VertBuf>>,
}

/// All the IBOs which a mesh might need to use.
pub struct MeshIBOs {
    pub tris: Rc<RefCell<gpu::IndexBuf>>,
    pub lines: Rc<RefCell<gpu::IndexBuf>>,
    pub points: Rc<RefCell<gpu::IndexBuf>>,

    /// IBOs per material, for material-specific draw calls
    pub tris_per_mat: HashMap<id::MaterialId, Rc<RefCell<gpu::IndexBuf>>>,
}

/// Contains data for all loops in the mesh
pub struct MeshBuffers {
    pub vbo: MeshVBOs,
    pub ibo: MeshIBOs,

    // Forces refreshing all buffers on `sync`
    is_dirty: bool,
}

/// A helper function for creation of shareable `gpu::VertBuf`s
fn _create_vbuf(prefix: &str, label: &str) -> Rc<RefCell<gpu::VertBuf>> {
    Rc::new(RefCell::new(gpu::VertBuf::new(format!("{prefix}.vbo.{label}"))))
}

/// A helper function for creation of shareable `gpu::IndexBuf`s
fn _create_ibuf(prefix: &str, label: &str) -> Rc<RefCell<gpu::IndexBuf>> {
    Rc::new(RefCell::new(gpu::IndexBuf::new(format!("{prefix}.ibo.{label}"))))
}

impl MeshBuffers {
    pub fn new(mesh: &pp_core::mesh::Mesh) -> Self {
        let mesh_lbl = mesh.label.as_str();
        Self {
            is_dirty: true,
            vbo: MeshVBOs {
                pos: _create_vbuf(mesh_lbl, "pos"),
                nor: _create_vbuf(mesh_lbl, "nor"),
                uv: _create_vbuf(mesh_lbl, "uv"),
                pos_2d: _create_vbuf(mesh_lbl, "pos_2d"),
                vert_idx: _create_vbuf(mesh_lbl, "vert_idx"),
                edge_idx: _create_vbuf(mesh_lbl, "edge_idx"),
                face_idx: _create_vbuf(mesh_lbl, "face_idx"),
            },
            ibo: MeshIBOs {
                tris: _create_ibuf(mesh_lbl, "tris"),
                points: _create_ibuf(mesh_lbl, "points"),
                lines: _create_ibuf(mesh_lbl, "lines"),
                tris_per_mat: HashMap::new(),
            },
        }
    }

    /// Synchronizes all cached buffers with the current state of the Mesh
    /// TODO: Sort out all this stuff
    pub fn sync(&mut self, ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh) {
        let dirty_flags = if self.is_dirty { &MeshDirtyFlags::all() } else { &mesh.elem_dirty };
        if dirty_flags.intersects(MeshDirtyFlags::VERTS) {
            self.extract_vbo_pos(ctx, mesh);
            self.extract_vbo_vnor(ctx, mesh);
        }
        if dirty_flags.intersects(MeshDirtyFlags::LOOPS) {
            self.extract_ibo_tris(ctx, mesh);
        }
        self.is_dirty = false
    }

    // --- Section: Extraction ---

    /// Reloads the pos VBO from the mesh's data
    fn extract_vbo_pos(&mut self, ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].po)
            .collect();
        self.vbo.pos.borrow_mut().update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    fn extract_vbo_vnor(&mut self, ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].no)
            .collect();
        self.vbo.nor.borrow_mut().update(ctx, data.as_slice());
    }

    /// Reloads the vertex VBO from the mesh's data
    fn extract_vbo_uv(&mut self, ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].no)
            .collect();
        self.vbo.nor.borrow_mut().update(ctx, data.as_slice());
    }

    // ---- Section: IBO Extraction ---
    fn extract_ibo_tris(&mut self, ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh) {
        let data: Vec<u32> = (0..mesh.faces.num_elements() * 3).map(|f| f as u32).collect();
        self.ibo.tris.borrow_mut().update(ctx, data.as_slice());
    }
}
