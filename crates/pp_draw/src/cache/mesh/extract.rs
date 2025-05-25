use bitflags::bitflags;

bitflags! {
    /// A mask of items to render for selection in the buffer
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct VertFlags: u32 {
        const SELECTED = 1 << 0;
        const ACTIVE = 1 << 1;
        const FACE_SELECTED = 1 << 2;
        const FACE_ACTIVE = 1 << 3;
    }

    /// A mask of items to render for selection in the buffer
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct EdgeFlags: u32 {
        const SELECTED = 1 << 0;
        const ACTIVE = 1 << 1;
        const V0_SELECTED = 1 << 2;
        const V1_SELECTED = 1 << 3;
        const CUT = 1 << 4;
    }
}

/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use cgmath::Transform;
    use pp_core::{
        id::{self, EdgeId, Id, LoopId},
        select::SelectionActiveElement,
    };

    use crate::{cache::mesh::extract::EdgeFlags, gpu};

    use super::VertFlags;

    /// Reloads the pos VBO from the mesh's data
    pub fn pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].po).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the piece pos VBOs, using their "unfolded" positions as determined
    /// by each piece's `t` value.
    pub fn piece_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_piece_faces_unfolded(id::PieceId::from_usize(p_id)))
            .flat_map(|item| {
                mesh.iter_face_loops(item.f).map(move |l| {
                    Into::<[f32; 3]>::into(
                        item.affine.transform_point(cgmath::Point3::from(mesh[mesh[l].v].po)),
                    )
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn edge_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> =
            mesh.edges.values().map(|e| [mesh[e.v[0]].po, mesh[e.v[1]].po]).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the piece vertex positons VBO from the mesh's data
    pub fn piece_edge_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> =
            mesh.pieces
                .indices()
                .flat_map(|p_id| mesh.iter_piece_faces_unfolded(id::PieceId::from_usize(p_id)))
                .flat_map(|item| {
                    mesh.iter_face_loops(item.f).map(move |l| {
                        [
                            Into::<[f32; 3]>::into(item.affine.transform_point(
                                cgmath::Point3::from(mesh[mesh[mesh[l].e].v[0]].po),
                            )),
                            Into::<[f32; 3]>::into(item.affine.transform_point(
                                cgmath::Point3::from(mesh[mesh[mesh[l].e].v[1]].po),
                            )),
                        ]
                    })
                })
                .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vnor(mesh: &pp_core::mesh::Mesh, l: LoopId) -> [f32; 3] {
        mesh[mesh[l].v].no
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _vnor(mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| mesh.iter_face_loops(f_id).map(|l| _vnor(mesh, l)))
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vert_flags(
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        l: LoopId,
    ) -> u32 {
        let mut flags = VertFlags::empty();
        if selection.faces.contains(&(mesh.id, mesh[l].f)) {
            flags |= VertFlags::FACE_SELECTED;
        }
        if selection.verts.contains(&(mesh.id, mesh[l].v)) {
            flags |= VertFlags::SELECTED;
        }
        if let Some(el) = selection.active_element.as_ref() {
            match el {
                SelectionActiveElement::Vert((m_id, v_id)) => {
                    if *m_id == mesh.id && *v_id == mesh[l].v {
                        flags |= VertFlags::ACTIVE;
                    }
                }
                SelectionActiveElement::Face((m_id, f_id)) => {
                    if *m_id == mesh.id && *f_id == mesh[l].f {
                        flags |= VertFlags::FACE_ACTIVE;
                    }
                }
                _ => {}
            }
        }
        flags.bits()
    }

    /// Reloads flags indicating the state of the vertex (select, active)
    pub fn vert_flags(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _vert_flags(mesh, selection, l)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vert_flags(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| mesh.iter_face_loops(f_id).map(|l| _vert_flags(mesh, selection, l)))
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _edge_flags(
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        e_id: EdgeId,
    ) -> u32 {
        let e = mesh[e_id];
        let mut flags = EdgeFlags::empty();
        if selection.edges.contains(&(mesh.id, e_id)) {
            flags |= EdgeFlags::SELECTED;
        }
        if selection.verts.contains(&(mesh.id, e.v[0])) {
            flags |= EdgeFlags::V0_SELECTED;
        }
        if selection.verts.contains(&(mesh.id, e.v[1])) {
            flags |= EdgeFlags::V1_SELECTED;
        }
        if e.is_cut {
            flags |= EdgeFlags::CUT;
        }
        flags.bits()
    }

    /// Reloads flags indicating the state of the vertex (select, active)
    pub fn edge_flags(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .edges
            .indices()
            .map(|e_id| _edge_flags(mesh, selection, id::EdgeId::from_usize(e_id)))
            .collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_edge_flags(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| {
                mesh.iter_face_loops(f_id).map(|l| _edge_flags(mesh, selection, mesh[l].e))
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vert_idx(mesh: &pp_core::mesh::Mesh, l: LoopId) -> [u32; 4] {
        [
            mesh[mesh[l].f].p.map(|p| p.idx() + 1).unwrap_or_default(), // `0` indicates no piece
            mesh[l].f.idx(),
            mesh[l].v.idx(),
            mesh.id.idx(),
        ]
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn vert_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _vert_idx(mesh, l)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vert_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| mesh.iter_face_loops(f_id).map(|l| _vert_idx(mesh, l)))
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _edge_idx(mesh: &pp_core::mesh::Mesh, e: usize) -> [u32; 2] {
        [e as u32, mesh.id.idx()]
    }

    /// Reloads the edge selection idx from the mesh
    pub fn edge_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.edges.indices().map(|e| _edge_idx(mesh, e)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn piece_edge_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| {
                mesh.iter_face_loops(f_id).map(|l| _edge_idx(mesh, mesh[l].e.to_usize()))
            })
            .collect();
        vbo.update(ctx, data.as_slice())
    }
}
