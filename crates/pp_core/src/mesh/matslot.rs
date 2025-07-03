/// A "slot" for a material in a mesh. This breaks the coupling between meshes
/// and materials and allows you to name specific usages of materials within a
/// mesh.
#[derive(Debug, Clone)]
pub struct MaterialSlot {
    pub label: String,
}
