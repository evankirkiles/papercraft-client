pub struct Mesh<'mesh> {
    pub label: Option<&'mesh str>,
    pub verts: Vec<MeshVertex>,
    // edges: Vec<MeshEdge>,
    // faces: Vec<MeshFace>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshFace {
    pub verts: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshEdge {
    pub verts: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
