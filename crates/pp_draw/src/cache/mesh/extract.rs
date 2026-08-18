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
        /// The edge's dihedral is convex, so it folds away from the viewer
        const MOUNTAIN = 1 << 6;
        /// The edge's dihedral is concave, so it folds toward the viewer
        const VALLEY = 1 << 7;
        /// A flap extends over this side of the edge. Per-loop, so this is only
        /// ever set in the piecewise VBO.
        const HAS_FLAP = 1 << 8;
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct EdgeFlapFlags: u32 {
        const EXISTS = 1 << 0;
    }
}

/// A flap's short edge, in the same unfolded piece-local space as the edge
/// positions VBO.
///
/// Only the two *top* corners travel: the bottom two are the flap's base edge,
/// which the edge positions VBO already carries. The trapezoid itself is shaped
/// by [`pp_core::mesh::flap::flap_corners`] on the CPU rather than in the vertex
/// shader, so the tab the printer strokes and the tab on screen are the same
/// four points.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct EdgeFlapInfo {
    pub top0: [f32; 3],
    pub top1: [f32; 3],
    pub flags: u32,
}

/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use cgmath::{EuclideanSpace, Transform};
    use pp_core::{
        id::{self, EdgeId, Id, LoopId},
        mesh::edge::FLAT_EDGE_ANGLE_EPSILON,
        select::SelectionActiveElement,
        MeshId,
    };
    use slotmap::Key;

    use crate::{cache::mesh::extract::EdgeFlags, gpu};

    use super::{EdgeFlapFlags, EdgeFlapInfo, VertFlags};

    /// Reloads the pos VBO from the mesh's data
    pub fn pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> =
            mesh.iter_loops().map(|l| Into::<[f32; 3]>::into(mesh.vert_pos(mesh[l].v))).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the piece pos VBOs, using their "unfolded" positions as determined
    /// by each piece's `t` value.
    pub fn piece_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .iter_pieces()
            .flat_map(|f_id| mesh.iter_piece_faces_unfolded(*f_id))
            .flat_map(|item| {
                mesh.iter_face_loops(item.f).map(move |l| {
                    Into::<[f32; 3]>::into(
                        item.affine
                            .transform_point(cgmath::Point3::from_vec(mesh.vert_pos(mesh[l].v))),
                    )
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn edge_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .edges
            .values()
            .map(|e| {
                [
                    Into::<[f32; 3]>::into(mesh.vert_pos(e.v[0])),
                    Into::<[f32; 3]>::into(mesh.vert_pos(e.v[1])),
                ]
            })
            .collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the piece vertex positons VBO from the mesh's data
    pub fn piece_edge_pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .iter_pieces()
            .flat_map(|f_id| mesh.iter_piece_faces_unfolded(*f_id))
            .flat_map(|item| {
                mesh.iter_face_loops(item.f).map(move |l| {
                    [
                        Into::<[f32; 3]>::into(item.affine.transform_point(
                            cgmath::Point3::from_vec(mesh.vert_pos(mesh[mesh[l].e].v[0])),
                        )),
                        Into::<[f32; 3]>::into(item.affine.transform_point(
                            cgmath::Point3::from_vec(mesh.vert_pos(mesh[mesh[l].e].v[1])),
                        )),
                    ]
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vnor(mesh: &pp_core::mesh::Mesh, l: LoopId) -> [f32; 3] {
        mesh[l].no
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _vnor(mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_piece_loops().map(|l| _vnor(mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _uv(mesh: &pp_core::mesh::Mesh, l: LoopId) -> [f32; 2] {
        mesh[l].uv
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn uv(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _uv(mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_uv(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_piece_loops().map(|l| _uv(mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vert_flags(
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        l: LoopId,
    ) -> u32 {
        let mut flags = VertFlags::empty();
        if selection.faces.contains(&(m_id, mesh[l].f)) {
            flags |= VertFlags::FACE_SELECTED;
        }
        if selection.verts.contains(&(m_id, mesh[l].v)) {
            flags |= VertFlags::SELECTED;
        }
        if let Some(el) = selection.active_element.as_ref() {
            match el {
                SelectionActiveElement::Vert(id) => {
                    if id.0 == m_id && id.1 == mesh[l].v {
                        flags |= VertFlags::ACTIVE;
                    }
                }
                SelectionActiveElement::Face(id) => {
                    if id.0 == m_id && id.1 == mesh[l].f {
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
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> =
            mesh.iter_loops().map(|l| _vert_flags(m_id, mesh, selection, l)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vert_flags(
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> =
            mesh.iter_piece_loops().map(|l| _vert_flags(m_id, mesh, selection, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _edge_flags(
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        e_id: EdgeId,
    ) -> u32 {
        let e = mesh[e_id];
        let id = (m_id, e_id);
        let mut flags = EdgeFlags::empty();
        if selection.edges.contains(&id) {
            flags |= EdgeFlags::SELECTED;
        }
        if selection.verts.contains(&(m_id, e.v[0])) {
            flags |= EdgeFlags::V0_SELECTED;
        }
        if selection.verts.contains(&(m_id, e.v[1])) {
            flags |= EdgeFlags::V1_SELECTED;
        }
        if mesh.edge_is_cut(&e_id) {
            flags |= EdgeFlags::CUT;
        }
        if e.l.is_none_or(|l| mesh[l].radial_next == l) {
            flags |= EdgeFlags::BORDER;
        }
        match mesh.edge_fold_angle(e_id) {
            Some(angle) if angle > FLAT_EDGE_ANGLE_EPSILON => flags |= EdgeFlags::MOUNTAIN,
            Some(angle) if angle < -FLAT_EDGE_ANGLE_EPSILON => flags |= EdgeFlags::VALLEY,
            _ => {}
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
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .edges
            .indices()
            .map(|e_id| _edge_flags(m_id, mesh, selection, id::EdgeId::from_usize(e_id)))
            .collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_edge_flags(
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        selection: &pp_core::select::SelectionState,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh
            .iter_piece_loops()
            .map(|l| {
                let mut flags = _edge_flags(m_id, mesh, selection, mesh[l].e);
                if mesh.loop_has_flap(l) {
                    flags |= EdgeFlags::HAS_FLAP.bits();
                }
                flags
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _vert_idx(m_id: MeshId, mesh: &pp_core::mesh::Mesh, l: LoopId) -> [u64; 2] {
        [((u64::from(mesh[l].v.idx())) << 32) | u64::from(mesh[l].f.idx()), m_id.data().as_ffi()]
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn vert_idx(
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh.iter_loops().map(|l| _vert_idx(m_id, mesh, l)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn piece_vert_idx(
        ctx: &gpu::Context,

        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh.iter_piece_loops().map(|l| _vert_idx(m_id, mesh, l)).collect();
        vbo.update(ctx, data.as_slice());
    }

    fn _edge_idx(m_id: MeshId, e: usize) -> [u64; 2] {
        [(e as u64) << 32, m_id.data().as_ffi()]
    }

    /// Reloads the edge selection idx from the mesh
    pub fn edge_idx(
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> = mesh.edges.indices().map(|e| _edge_idx(m_id, e)).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Reloads the vertex selection idx from the mesh
    pub fn piece_edge_idx(
        ctx: &gpu::Context,
        m_id: MeshId,
        mesh: &pp_core::mesh::Mesh,
        vbo: &mut gpu::VertBuf,
    ) {
        let data: Vec<_> =
            mesh.iter_piece_loops().map(|l| _edge_idx(m_id, mesh[l].e.to_usize())).collect();
        vbo.update(ctx, data.as_slice())
    }

    /// Extracts the flap hanging over each of the piece's edge loops: the two
    /// top corners of its trapezoid, and whether it exists at all.
    ///
    /// The shape itself comes from `pp_core`, which owns the flap geometry so the
    /// printed tab and the on-screen tab agree; this only walks the loops and
    /// flattens the result into the buffer.
    pub fn piece_edge_flap(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .iter_pieces()
            .flat_map(|f_id| {
                let walker = mesh.iter_piece_faces_unfolded(*f_id);
                let t = walker.t;
                walker.flat_map(move |item| {
                    mesh.iter_face_loops(item.f).map(move |l_id| {
                        // A loop with no flap keeps the default, whose cleared
                        // EXISTS flag the shader clips away.
                        mesh.piece_flap_corners(l_id, item.affine, t)
                            .map(|corners| EdgeFlapInfo {
                                top0: corners[3].into(),
                                top1: corners[2].into(),
                                flags: EdgeFlapFlags::EXISTS.bits(),
                            })
                            .unwrap_or_default()
                    })
                })
            })
            .collect();
        vbo.update(ctx, data.as_slice());
    }
}

pub mod ibo {
    use pp_core::{
        id::{self},
        MaterialId,
    };
    use slotmap::SecondaryMap;

    use crate::{cache::mesh::MaterialGPUVBORange, gpu};

    /// Gets an ordered IBO for rendering materials. We sort it so each material's
    /// surface tris can be drawn from a contiguous range within this IBO.
    pub fn mat_indices(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        default_mat: &MaterialId,
        ibo: &mut gpu::IndexBuf,
        ranges: &mut SecondaryMap<MaterialId, MaterialGPUVBORange>,
    ) {
        let mut data: Vec<_> = mesh
            .iter_loops()
            .zip(0u32..)
            .map(|(l, i)| (i, mesh[mesh[l].f].m.unwrap_or(*default_mat)))
            .collect();
        data.sort_by(|(_, m_a), (_, m_b)| m_a.cmp(m_b));
        let mut i_prev: u32 = 0;
        let mut m_prev: Option<MaterialId> = None;
        let final_data: Vec<_> = data
            .iter()
            .zip(0u32..)
            .map(|((ibo_i, m_id), i)| {
                // If we've changed materials, update i_prev to begin at the new material
                if m_prev.is_some_and(|m_prev| m_prev != *m_id) {
                    i_prev = i;
                };
                let m = ranges.entry(*m_id).unwrap().or_default();
                m.range = i_prev..(i + 1);
                m_prev = Some(*m_id);
                *ibo_i
            })
            .collect();
        ibo.update(ctx, final_data.as_slice());
    }

    /// Gets an ordered IBO for rendering materials. We sort it so each material's
    /// face indices occupy contiguous blocks and piece indices occupy contiguous,
    /// blocks within that, allowing us to cut down on material binds and have more
    /// frequent piece binds (driven by a uniform).
    /// E.g.:
    /// |    M1    |    M2    |
    /// | P1 P2 P3 | P1 P2 P3 |
    pub fn piece_mat_indices(
        ctx: &gpu::Context,
        mesh: &pp_core::mesh::Mesh,
        default_mat: &MaterialId,
        ibo: &mut gpu::IndexBuf,
        mats: &mut SecondaryMap<MaterialId, MaterialGPUVBORange>,
    ) {
        let mut data: Vec<_> = mesh
            .iter_pieces()
            .flat_map(|p_id| mesh.iter_connected_faces(*p_id).map(move |f_id| (f_id, *p_id)))
            .flat_map(|(f_id, p_id)| mesh.iter_face_loops(f_id).map(move |l_id| (l_id, p_id)))
            .zip(0u32..)
            .map(|((l, p), i)| (i, mesh[mesh[l].f].m.unwrap_or(*default_mat), p))
            .collect();
        data.sort_by(|(_, m_a, p_a), (_, m_b, p_b)| m_a.cmp(m_b).then(p_a.cmp(p_b)));
        let mut i_prev: u32 = 0;
        let mut m_prev: Option<MaterialId> = None;
        let mut p_prev: Option<id::FaceId> = None;
        // Clear out all the existing piece_ranges
        mats.iter_mut().for_each(|(_, mat)| mat.piece_ranges.clear());
        let final_data: Vec<_> = data
            .iter()
            .zip(0u32..)
            .map(|((ibo_i, m_id, p_id), i)| {
                // Ensure the `Material` entries are up-to-date
                if m_prev.is_some_and(|m_prev| m_prev != *m_id)
                    || p_prev.is_some_and(|p_prev| p_prev != *p_id)
                {
                    i_prev = i;
                };
                let m = mats.entry(*m_id).unwrap().or_default();
                m.piece_ranges.insert(*p_id, i_prev..(i + 1));
                m_prev = Some(*m_id);
                p_prev = Some(*p_id);
                *ibo_i
            })
            .collect();
        ibo.update(ctx, final_data.as_slice());
    }
}
