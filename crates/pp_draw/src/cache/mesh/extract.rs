/// Helper functions for extracting VBOs from a Mesh
pub mod vbo {
    use crate::gpu;
    use pp_core::id::{self, Id};

    /// Reloads the pos VBO from the mesh's data
    pub fn pos(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].po)
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn vnor(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].no)
            .collect();
        vbo.update(ctx, data.as_slice());
    }

    /// Reloads the vertex normals VBO from the mesh's data
    pub fn uv(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, vbo: &mut gpu::VertBuf) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .flat_map(|f| mesh.face_loop_walk(id::FaceId::from_usize(f)))
            .flat_map(|l| mesh[mesh[l].v].no)
            .collect();
        vbo.update(ctx, data.as_slice());
    }
}

/// Helper functions for extracting IBOs from a Mesh
pub mod ibo {
    use crate::gpu;
    use pp_core::id::{self, Id};

    /// Reloads the pos VBO from the mesh's data
    pub fn tris(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, ibo: &mut gpu::IndexBuf) {
        let data: Vec<u32> = (0..mesh.faces.num_elements() * 3).map(|f| f as u32).collect();
        ibo.update(ctx, data.as_slice());
    }

    /// Reloads the pos VBO from the mesh's data
    pub fn lines(ctx: &gpu::Context, mesh: &pp_core::mesh::Mesh, ibo: &mut gpu::IndexBuf) {
        let data: Vec<_> = mesh
            .faces
            .indices()
            .enumerate()
            .flat_map(|(f_i, f)| {
                let f_len = 3;
                mesh.face_loop_walk(id::FaceId::from_usize(f))
                    .enumerate()
                    .flat_map(move |(l_i, _)| [f_i + l_i, f_i + ((l_i + 1) % f_len)])
            })
            .map(|f| f as u32)
            .collect();
        ibo.update(ctx, data.as_slice());
    }
}
