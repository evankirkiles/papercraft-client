use crate::gpu;
use material::{image::ImageGPU, texture::TextureGPU, MaterialGPU};
use pp_core::id;
use std::collections::HashMap;

pub(crate) use mesh::MeshGPU;

mod common;
mod material;
mod mesh;
mod viewport_2d;
mod viewport_3d;

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
    pub viewport_3d: viewport_3d::Viewport3DGPU,
    pub viewport_2d: viewport_2d::Viewport2DGPU,
    pub common: common::CommonGPUResources,
}

impl DrawCache {
    pub(crate) fn new(ctx: &gpu::Context) -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            images: HashMap::new(),
            viewport_3d: viewport_3d::Viewport3DGPU::new(ctx),
            viewport_2d: viewport_2d::Viewport2DGPU::new(ctx),
            common: common::CommonGPUResources::new(ctx),
        }
    }

    pub(crate) fn sync_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure state meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(key, mesh)| {
            let m = self.meshes.entry(*key).or_insert(MeshGPU::new(mesh));
            m.sync(ctx, mesh, &state.selection);
        });
        state.selection.is_dirty = false;
        // TODO: Remove unused meshes from the DrawCache
    }

    pub(crate) fn sync_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
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

    pub(crate) fn sync_viewports(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        let (width, height) = (ctx.config.width as f32, ctx.config.height as f32);
        let x_border = state.settings.viewport_split_x as f32 * width;
        self.viewport_3d.sync(ctx, &state.viewport_3d, 0.0, 0.0, x_border, height);
        self.viewport_2d.sync(ctx, &state.viewport_2d, x_border, 0.0, width - x_border, height);
    }
}
