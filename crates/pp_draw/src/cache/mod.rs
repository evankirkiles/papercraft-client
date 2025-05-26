use crate::gpu;
use pp_core::id;
use pp_core::mesh::MeshElementType;
use std::collections::HashMap;

pub(crate) use mesh::MeshGPU;

mod common;
mod mesh;
mod viewport_2d;
mod viewport_3d;

/// A manager for image textures for materials
#[derive(Debug)]
pub(crate) struct MaterialGPU {}

/// Represents the current state of all allocated GPU resources.
///
/// On "sync", the state of the DrawCache is diff'ed against the App State.
/// This is done in three steps:
///   1. For each AppState item (meshes.for_each), ensure it is present and up-to-date in DrawCache
///   2. For each DrawCache item (meshes.for_each), if it's not in the AppState, remove it
#[derive(Debug)]
pub(crate) struct DrawCache {
    pub meshes: HashMap<id::MeshId, MeshGPU>,
    pub materials: HashMap<id::MaterialId, MaterialGPU>,
    pub viewport_3d: viewport_3d::Viewport3DGPU,
    pub viewport_2d: viewport_2d::Viewport2DGPU,
    pub common: common::CommonGPUResources,
}

impl DrawCache {
    pub(crate) fn new(ctx: &gpu::Context) -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            viewport_3d: viewport_3d::Viewport3DGPU::new(ctx),
            viewport_2d: viewport_2d::Viewport2DGPU::new(ctx),
            common: common::CommonGPUResources::new(ctx),
        }
    }

    pub(crate) fn sync_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure AppState's meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(key, mesh)| {
            mesh.ensure_elem_index(MeshElementType::all());
            let m = self.meshes.entry(*key).or_insert(MeshGPU::new(mesh));
            m.sync(ctx, mesh, &state.selection);
        });
        state.selection.is_dirty = false;
        // TODO: Remove unused meshes from the DrawCache
    }

    pub(crate) fn sync_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {}

    pub(crate) fn sync_viewports(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        let (width, height) = (ctx.config.width as f32, ctx.config.height as f32);
        let x_border = state.settings.viewport_split_x as f32 * width;
        self.viewport_3d.sync(ctx, &state.viewport_3d, 0.0, 0.0, x_border, height);
        self.viewport_2d.sync(ctx, &state.viewport_2d, x_border, 0.0, width - x_border, height);
    }
}
