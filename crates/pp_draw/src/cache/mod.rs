use crate::gpu;
use material::{image::ImageGPU, texture::TextureGPU, MaterialGPU};
use pp_core::id;
use pp_editor::ViewportId;
use slotmap::SecondaryMap;
use std::collections::HashMap;
use viewport::{BindableViewport, ViewportGPU};

pub(crate) use mesh::MeshGPU;

mod common;
mod material;
mod mesh;
pub mod viewport;

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
    pub textures: HashMap<id::TextureId, TextureGPU>,
    pub images: HashMap<id::ImageId, ImageGPU>,
    pub viewports: SecondaryMap<ViewportId, ViewportGPU>,
    pub common: common::CommonGPUResources,
}

impl DrawCache {
    pub(crate) fn new(ctx: &gpu::Context) -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            images: HashMap::new(),
            viewports: SecondaryMap::new(),
            common: common::CommonGPUResources::new(ctx),
        }
    }

    pub(crate) fn prepare_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure state meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(key, mesh)| {
            let m = self.meshes.entry(*key).or_insert(MeshGPU::new(mesh));
            m.sync(ctx, mesh, &state.selection);
        });
        state.selection.is_dirty = false;
        // TODO: Remove unused meshes from the DrawCache
    }

    pub(crate) fn prepare_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure all images have been uploaded to the GPU
        state.images.iter_mut().for_each(|(key, img)| {
            self.images.entry(*key).or_insert(ImageGPU::new(ctx, img));
        });

        // Create all textures (image views / samplers to put in material bind groups)
        state.textures.iter_mut().for_each(|(key, tex)| {
            let image = &self.images[&tex.image];
            self.textures.entry(*key).or_insert(TextureGPU::new(ctx, tex, image));
        });

        // Finally, ensure all the materials are set up (the bind groups themselves)
        state.materials.iter_mut().for_each(|(key, mat)| {
            if !self.materials.contains_key(key) || mat.is_dirty {
                self.materials.insert(*key, MaterialGPU::new(ctx, mat, &self.textures));
            }
            mat.is_dirty = false;
        });
    }

    /// Ensures that all draw cache viewports have been synchronized
    pub(crate) fn prepare_viewports(&mut self, ctx: &gpu::Context, editor: &mut pp_editor::Editor) {
        editor
            .viewports
            .iter_mut()
            .filter(|(_, viewport)| viewport.bounds.area.has_area())
            .for_each(|(key, viewport)| {
                self.viewports
                    .entry(key)
                    .unwrap()
                    .or_insert_with(|| ViewportGPU::new(ctx, viewport))
                    .sync(ctx, viewport)
                    .expect("Viewport type changed!");
            });
    }
}
