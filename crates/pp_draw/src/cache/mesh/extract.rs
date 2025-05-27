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
        const BORDER = 1 << 5;
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct EdgeFlapFlags: u32 {
        const EXISTS = 1 << 0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct EdgeFlapInfo {
    pub v3_pos: [f32; 3],
    pub flags: u32,
}

/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use cgmath::{InnerSpace, Matrix4, Rad, Transform, Vector3};
    use pp_core::{
        id::{self, EdgeId, Id, LoopId},
        select::SelectionActiveElement,
    };

    use crate::{cache::mesh::extract::EdgeFlags, gpu};

    use super::{EdgeFlapFlags, EdgeFlapInfo, VertFlags};

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
        let id = (mesh.id, e_id);
        let mut flags = EdgeFlags::empty();
        if selection.edges.contains(&id) {
            flags |= EdgeFlags::SELECTED;
        }
        if selection.verts.contains(&(mesh.id, e.v[0])) {
            flags |= EdgeFlags::V0_SELECTED;
        }
        if selection.verts.contains(&(mesh.id, e.v[1])) {
            flags |= EdgeFlags::V1_SELECTED;
        }
        if e.cut.is_some() {
            flags |= EdgeFlags::CUT;
        }
        if selection.active_element.as_ref().is_some_and(|el| match el {
            SelectionActiveElement::Edge(active_id) => id == *active_id,
            _ => false,
        }) {
            flags |= EdgeFlags::ACTIVE;
        };
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

    /// Extracts flap-specific information from the edges in the piece. This information is:
    ///  - flags (whether or not the flap exists, any other info we need to pack in here)
    ///  - v3_pos (the position of the anchor vertex the piece should reach to)
    pub fn piece_edge_flap(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .pieces
            .indices()
            .flat_map(|p_id| {
                let walker = mesh.iter_piece_faces_unfolded(id::PieceId::from_usize(p_id));
                let t = walker.t;
                walker.flat_map(move |item| {
                    mesh.iter_face_loops(item.f).map(move |l_id| {
                        let l = mesh[l_id];
                        let e = mesh[l.e];
                        let l_2 = mesh[l.radial_next];

                        // If edge is not cut or there's no flap here, just return the default,
                        // which will not render / render an invisible flap
                        if e.cut.is_none_or(|e| e.l_flap.is_none_or(|l_id_2| l_id == l_id_2))
                            // Or if this edge doesn't have another face
                            || l_id == l.radial_next
                        {
                            return EdgeFlapInfo {
                                v3_pos: [0.0, 0.0, 0.0],
                                flags: EdgeFlapFlags::empty().bits(),
                            };
                        }

                        // Here, we need the unfolded position of the vertex across
                        // the cut boundary of the current edge (if cut). We use
                        // this as the anchor to determine the positions of the
                        // vertices on the short edge of the flap.
                        let (v0_id, v1_id) = (e.v[0], e.v[1]);

                        // 1. Get current positions of v0, v1 in untransformed space
                        // to determine the shared edge axis we need to rotate face 2 around
                        let v0 = Vector3::from(mesh[v0_id].po);
                        let v1 = Vector3::from(mesh[v1_id].po);
                        let axis = (v1 - v0).normalize();

                        // 2. Compare face normals to determine the angle we need to rotate face 2
                        // by (around the shared edge) to make it coplanar with face 1
                        let n1 = Vector3::from(mesh[l.f].no);
                        let n2 = Vector3::from(mesh[l_2.f].no);
                        // Compute angle to rotate n2 onto n1 around `axis`
                        let cross = n2.cross(n1); // direction from n2 to n1
                        let dot = n2.dot(n1);
                        let angle = axis.dot(cross).atan2(dot) * t; // signed angle from n2 to n1

                        // 3. Create affine transform to rotate all vertices in face 2
                        // by `angle` around the shared edge, bringing face 2 onto the same
                        // plane to the "root" face.
                        let translation_origin = Matrix4::from_translation(-v0);
                        let rotation = Matrix4::from_axis_angle(axis, Rad(angle));
                        let translation_back = Matrix4::from_translation(v0);
                        let local_rotation = translation_back * rotation * translation_origin;
                        let affine = item.affine * local_rotation;

                        // The point we want is the single vertex in f_2 which is neither v0 nor v1.
                        // We need to apply the final calcualted transformation to it.
                        let v3 = mesh
                            .iter_face_loops(l_2.f)
                            .map(|l| mesh[l].v)
                            .find(|v| *v != v0_id && *v != v1_id)
                            .unwrap();
                        let v3_pos = affine.transform_point(cgmath::Point3::from(mesh[v3].po));

                        // Here, we have the face and correct vertex positions.
                        EdgeFlapInfo { v3_pos: v3_pos.into(), flags: EdgeFlapFlags::EXISTS.bits() }
                    })
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }
}
