/// Contains all the
pub struct MeshBatchList {}

pub struct MeshBatchCache {
    batch: MeshBatchList,

    // TODO: Per-material IBOs (and Surfaces?)
    tris_per_mat: Vec<u32>,
    surface_per_mat: Vec<u32>,
}
