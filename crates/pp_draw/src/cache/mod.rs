use std::collections::HashMap;

use crate::gpu;
use pp_core::id;

mod mesh;
mod viewport;

pub use mesh::batches::batch_buffer_layouts;
pub use mesh::MeshGPU;
use pp_core::mesh::MeshDirtyFlags;
pub use viewport::ViewportGPU;

/// A manager for image textures for materials
pub struct MaterialGPU {}

/// Represents the current state of all allocated GPU resources.
///
/// On "sync", the state of the DrawCache is diff'ed against the App State.
/// This is done in three steps:
///   1. For each AppState item (meshes.for_each), ensure it is present and up-to-date in DrawCache
///   2. For each DrawCache item (meshes.for_each), if it's not in the AppState, remove it
#[derive(Default)]
pub struct DrawCache {
    pub meshes: HashMap<id::MeshId, MeshGPU>,
    pub materials: HashMap<id::MaterialId, MaterialGPU>,
    pub viewports: HashMap<id::ViewportId, ViewportGPU>,
}

impl DrawCache {
    pub fn sync_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {
        // Ensure AppState's meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(key, mesh)| {
            let m = self.meshes.entry(*key).or_insert(MeshGPU::new(mesh));
            m.bufs.sync(ctx, mesh);
            mesh.elem_dirty = MeshDirtyFlags::empty();
            mesh.index_dirty = MeshDirtyFlags::empty();
        });
        // TODO: Remove unused meshes from the DrawCache
    }

    pub fn sync_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {}

    pub fn sync_viewports(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {
        // Ensure AppState's viwwports are all synced in the DrawCache
        state.viewports.iter_mut().for_each(|(key, viewport)| {
            let m = self.viewports.entry(*key).or_insert(ViewportGPU::new(ctx));
            m.sync(ctx, viewport);
            viewport.camera.is_dirty = false
        });
        // TODO: Remove viewports from the DrawCache
    }
}