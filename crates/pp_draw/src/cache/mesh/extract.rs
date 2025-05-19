use bitflags::bitflags;

bitflags! {
    /// A mask of items to render for selection in the buffer
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct VertFlags: u32 {
        const SELECTED = 1 << 0;
        const ACTIVE = 1 << 1;
        const FACE_SELECTED = 1 << 2;
    }

    /// A mask of items to render for selection in the buffer
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct EdgeFlags: u32 {
        const SELECTED = 1 << 0;
        const ACTIVE = 1 << 1;
        const V0_SELECTED = 1 << 2;
        const V1_SELECTED = 1 << 3;
    }
}

/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use cgmath::Transform;
    use pp_core::{
        id::{self, Id},
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

    pub fn cut_edge_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .edges
            .values()
            .filter(|e| e.is_cut)
            .map(|e| [mesh[e.v[0]].po, mesh[e.v[1]].po])
            .collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].no).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| mesh.iter_connected_faces(mesh[id::PieceId::from_usize(p_id)].f))
            .flat_map(|f_id| mesh.iter_face_loops(f_id).map(|l| [0.0, 0.0, 0.0]))
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    // /// Reloads the vertex normals VBO from the mesh's data
    // pub fn uv(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
    //     let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].no).collect();
    //     vbo.update(ctx, data.as_slice());
    // }

    /// Reloads flags indicating the state of the vertex (select, active)
    pub fn vert_flags(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .iter_loops()
            .map(|l| {
                let mut flags = VertFlags::empty();
                if selection.faces.contains(&(mesh.id, mesh[l].f)) {
                    flags |= VertFlags::FACE_SELECTED;
                }
                if selection.verts.contains(&(mesh.id, mesh[l].v)) {
                    flags |= VertFlags::SELECTED;
                }
                if selection.active_element.as_ref().is_some_and(|el| match el {
                    SelectionActiveElement::Vert((m_id, v_id)) => {
                        *m_id == mesh.id && *v_id == mesh[l].v
                    }
                    _ => false,
                }) {
                    flags |= VertFlags::ACTIVE;
                }
                flags.bits()
            })
            .collect();
        vbo.update(ctx, data.as_slice())
    }
    ///
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
            .flat_map(|f_id| {
                mesh.iter_face_loops(f_id).map(|l| {
                    let mut flags = VertFlags::empty();
                    if selection.faces.contains(&(mesh.id, mesh[l].f)) {
                        flags |= VertFlags::FACE_SELECTED;
                    }
                    if selection.verts.contains(&(mesh.id, mesh[l].v)) {
                        flags |= VertFlags::SELECTED;
                    }
                    if selection.active_element.as_ref().is_some_and(|el| match el {
                        SelectionActiveElement::Vert((m_id, v_id)) => {
                            *m_id == mesh.id && *v_id == mesh[l].v
                        }
                        _ => false,
                    }) {
                        flags |= VertFlags::ACTIVE;
                    }
                    flags.bits()
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
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
            .iter()
            .map(|(e_id, e)| {
                let mut flags = EdgeFlags::empty();
                if selection.edges.contains(&(mesh.id, id::EdgeId::from_usize(e_id))) {
                    flags |= EdgeFlags::SELECTED;
                }
                if selection.verts.contains(&(mesh.id, e.v[0])) {
                    flags |= EdgeFlags::V0_SELECTED;
                }
                if selection.verts.contains(&(mesh.id, e.v[1])) {
                    flags |= EdgeFlags::V1_SELECTED;
                }
                flags.bits()
            })
            .collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn vert_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| [mesh.id.idx(), mesh[l].v.idx()]).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn edge_idx(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.edges.indices().map(|e| [mesh.id.idx(), e as u32]).collect();
        vbo.update(ctx, data.as_slice())
    }
}
