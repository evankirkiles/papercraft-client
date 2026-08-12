use super::MeshElementType;

impl super::Mesh {
    /// Applies an incremental affine transform (translate + rotate) to the mesh.
    /// Only affects the whole-mesh 3D view; pieces are unaffected.
    pub fn transform_mesh(&mut self, delta: cgmath::Matrix4<f32>) {
        self.transform = delta * self.transform;
        self.uniform_dirty = true;
    }

    /// Applies an incremental uniform scale factor to the mesh, across all
    /// axes. Affects the mesh's own geometry as well as its derived pieces.
    pub fn scale_mesh(&mut self, factor: f32) {
        self.scale *= factor;
        self.elem_dirty |= MeshElementType::VERTS | MeshElementType::PIECES;
    }
}
