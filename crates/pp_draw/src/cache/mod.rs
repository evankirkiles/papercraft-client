use crate::gpu;

use pp_core::id;
use pp_core::mesh::MeshElementType;
use std::collections::HashMap;

use prelude::*;
pub mod prelude;

mod mesh;
mod viewport;

pub use mesh::MeshGPU;
pub use viewport::ViewportGPU;

/// A manager for image textures for materials
#[derive(Debug)]
pub struct MaterialGPU {}

/// Represents the current state of all allocated GPU resources.
///
/// On "sync", the state of the DrawCache is diff'ed against the App State.
/// This is done in three steps:
///   1. For each AppState item (meshes.for_each), ensure it is present and up-to-date in DrawCache
///   2. For each DrawCache item (meshes.for_each), if it's not in the AppState, remove it
#[derive(Debug)]
pub struct DrawCache {
    pub meshes: HashMap<id::MeshId, MeshGPU>,
    pub materials: HashMap<id::MaterialId, MaterialGPU>,
    pub viewport_3d: viewport::Viewport3DGPU,
    pub viewport_2d: viewport::Viewport2DGPU,
}

impl DrawCache {
    pub fn new(ctx: &gpu::Context) -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            viewport_3d: viewport::Viewport3DGPU::new(ctx),
            viewport_2d: viewport::Viewport2DGPU::new(ctx),
        }
    }

    pub fn sync_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {
        // Ensure AppState's meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(key, mesh)| {
            mesh.ensure_elem_index(MeshElementType::all());
            let m = self.meshes.entry(*key).or_insert(MeshGPU::new(ctx, mesh));
            m.sync(ctx, mesh);
            mesh.elem_dirty = MeshElementType::empty();
            mesh.index_dirty = MeshElementType::empty();
        });
        // TODO: Remove unused meshes from the DrawCache
    }

    pub fn sync_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {}

    pub fn sync_viewports(&mut self, ctx: &gpu::Context, state: &mut pp_core::state::State) {
        let (width, height) = (ctx.config.width as f32, ctx.config.height as f32);
        let x_border = state.viewport_split * width;
        self.viewport_3d.sync(ctx, &state.viewport_3d, 0.0, 0.0, x_border, height);
        self.viewport_2d.sync(ctx, &state.viewport_2d, x_border, 0.0, width - x_border, height);
    }
}
