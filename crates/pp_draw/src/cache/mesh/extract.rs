use bitflags::bitflags;

bitflags! {
    /// A mask of items to render for selection in the buffer
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct SelectionMask: u32 {
        const SELECTED = 1 << 0;
        const ACTIVE = 1 << 1;
    }
}

/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use pp_core::id::Id;

    use crate::gpu;

    use super::SelectionMask;

    /// Reloads the pos VBO from the mesh's data
    pub fn pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].po).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].no).collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn uv(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[mesh[l].v].no).collect();
        vbo.update(ctx, data.as_slice());
    }

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
                let mut flags = SelectionMask::empty();
                if selection.verts.contains(&(mesh.id, mesh[l].v)) {
                    flags |= SelectionMask::SELECTED;
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

    /// Flags used for rendering select state / active state
    pub fn edit_data(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {}
}

/// Helper functions for extracting IBOs from a Mesh
pub mod ibo {
    use crate::gpu;

    /// Reloads the IBO for tris from the mesh's data
    pub fn tris(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, ibo: &mut gpu::IndexBuf) {
        let data: Vec<_> = mesh.iter_loops().map(|l| mesh[l].index.unwrap() as u32).collect();
        ibo.update(ctx, data.as_slice());
    }

    /// Reloads the IBO for lines from the mesh's data
    pub fn lines(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, ibo: &mut gpu::IndexBuf) {
        let data: Vec<_> = mesh
            .edges
            .values()
            .filter_map(|e| e.l)
            .flat_map(|l| {
                let l = mesh[l];
                let l_next = mesh[l.next];
                [l.index.unwrap() as u32, l_next.index.unwrap() as u32]
            })
            .collect();
        ibo.update(ctx, data.as_slice());
    }
}
