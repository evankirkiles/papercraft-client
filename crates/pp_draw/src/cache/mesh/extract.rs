/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use crate::gpu;

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
