use crate::gpu;
use material::{image::ImageGPU, sampler::SamplerGPU, texture::TextureGPU, MaterialGPU};
use pp_core::{ImageId, MaterialId, MeshId, SamplerId, TextureId};
use pp_editor::{tool::Tool, ViewportId};
use slotmap::SecondaryMap;
use tool::{ActiveToolGPU, ToolGPU};
use viewport::{BindableViewport, ViewportGPU};

pub(crate) use mesh::MeshGPU;

mod common;
pub mod material;
pub mod mesh;
pub mod tool;
pub mod viewport;

/// Represents the current state of all allocated GPU resources.
///
/// On "sync", the state of the DrawCache is diff'ed against the App State.
/// This is done in three steps:
///   1. For each AppState item (meshes.for_each), ensure it is present and up-to-date in DrawCache
///   2. For each DrawCache item (meshes.for_each), if it's not in the AppState, remove it
#[derive(Debug)]
pub(crate) struct DrawCache {
    // State-specific GPU resources
    pub meshes: SecondaryMap<MeshId, MeshGPU>,
    pub materials: SecondaryMap<MaterialId, MaterialGPU>,
    pub textures: SecondaryMap<TextureId, TextureGPU>,
    pub samplers: SecondaryMap<SamplerId, SamplerGPU>,
    pub images: SecondaryMap<ImageId, ImageGPU>,

    // Editor-specific GPU resources
    pub viewports: SecondaryMap<ViewportId, ViewportGPU>,
    pub active_tool: Option<ActiveToolGPU>,

    /// Shared GPU resources
    pub common: common::CommonGPUResources,
}

impl DrawCache {
    pub(crate) fn new(ctx: &gpu::Context) -> Self {
        Self {
            meshes: SecondaryMap::new(),
            materials: SecondaryMap::new(),
            samplers: SecondaryMap::new(),
            textures: SecondaryMap::new(),
            images: SecondaryMap::new(),
            viewports: SecondaryMap::new(),
            common: common::CommonGPUResources::new(ctx),
            active_tool: None,
        }
    }

    pub(crate) fn prepare_meshes(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure state meshes are all synced in the DrawCache
        state.meshes.iter_mut().for_each(|(m_id, mesh)| {
            self.meshes.entry(m_id).unwrap().or_insert(MeshGPU::new(mesh)).sync(
                ctx,
                m_id,
                mesh,
                state.mesh_materials.get(m_id).unwrap(),
                &state.defaults.material,
                &state.selection,
            );
        });

        state.selection.is_dirty = false;
        // TODO: Remove unused meshes from the DrawCache
    }

    pub(crate) fn prepare_materials(&mut self, ctx: &gpu::Context, state: &mut pp_core::State) {
        // Ensure all images have been uploaded to the GPU
        state.images.iter_mut().for_each(|(i_id, img)| {
            self.images.entry(i_id).unwrap().or_insert(ImageGPU::new(ctx, img));
        });

        // Create all textures (image views / samplers to put in material bind groups)
        state.samplers.iter_mut().for_each(|(s_id, sampler)| {
            self.samplers.entry(s_id).unwrap().or_insert(SamplerGPU::new(ctx, sampler));
        });

        // Create all textures (image views / samplers to put in material bind groups)
        state.textures.iter_mut().for_each(|(t_id, tex)| {
            self.textures.entry(t_id).unwrap().or_insert(TextureGPU::new(tex));
        });

        // Finally, ensure all the materials are set up (the bind groups themselves)
        state.materials.iter_mut().for_each(|(m_id, mat)| {
            if !self.materials.contains_key(m_id) || mat.is_dirty {
                self.materials.insert(
                    m_id,
                    MaterialGPU::new(ctx, mat, &self.textures, &self.images, &self.samplers),
                );
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

    /// Ensures that the active tool is synchronized with its current state
    pub(crate) fn prepare_tool(&mut self, ctx: &gpu::Context, editor: &mut pp_editor::Editor) {
        let Some(viewport) = &editor.active_viewport else {
            self.active_tool = None;
            return;
        };
        match (&editor.active_tool, &mut self.active_tool) {
            (Some(tool), Some(tool_gpu)) => match (tool, &mut tool_gpu.tool) {
                // Map corresponding CPU-side tools to GPU-side tool resources
                (Tool::SelectBox(tool), ToolGPU::SelectBox(tool_gpu)) => tool_gpu.sync(ctx, tool),
                // If we didn't have the same type of tool, then reset the tool
                _ => {
                    let viewport = *viewport;
                    self.active_tool =
                        Some(ActiveToolGPU { viewport, tool: ToolGPU::new(ctx, tool) })
                }
            },
            (Some(tool), None) => {
                let viewport = *viewport;
                self.active_tool = Some(ActiveToolGPU { viewport, tool: ToolGPU::new(ctx, tool) })
            }
            (None, Some(_)) => self.active_tool = None,
            (None, None) => {}
        };
    }
}
