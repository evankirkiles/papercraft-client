use crate::{standard, SaveFile};
use pp_core::{material::texture::Texture, State};
use thiserror::Error;

/// Possible errors that can occur while loading a file
#[derive(Debug, Clone, Copy, Error)]
pub enum LoadError {
    #[error("unknown data store error")]
    Unknown,
    #[error("failed to load buffers")]
    FailedToLoadBuffers,
}

pub trait Loadable {
    fn load(save: SaveFile) -> Result<pp_core::State, LoadError>;
}

impl Loadable for pp_core::State {
    fn load(save: SaveFile) -> Result<pp_core::State, LoadError> {
        let mut state = State::default();

        // Extract buffer data (basically, all geometry data) out of the GLTF
        let mut gltf = save.0;
        let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob.take())
            .map_err(|_| LoadError::FailedToLoadBuffers)?;

        // Step 1: Load images
        let image_ids: Vec<_> = gltf
            .images()
            .enumerate()
            .map(|(i, gltf_image)| {
                state.images.insert(standard::image::load_image(&gltf_image, &buffers, i))
            })
            .collect();

        // Step 2: Load samplers
        let sampler_ids: Vec<_> = gltf
            .samplers()
            .map(|gltf_samp| state.samplers.insert(standard::sampler::load_sampler(&gltf_samp)))
            .collect();

        // Step 3: Load textures (combos of image + sampler)
        let default_texture = state.defaults.texture;
        let texture_ids: Vec<_> = gltf
            .textures()
            .enumerate()
            .map(|(i, gltf_texture)| {
                state.textures.insert(Texture {
                    label: gltf_texture
                        .name()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| format!("Tex{}", i)),
                    image: *image_ids
                        .get(gltf_texture.source().index())
                        .unwrap_or(&state.defaults.image),
                    sampler: gltf_texture
                        .sampler()
                        .index()
                        .map(|id| *sampler_ids.get(id).unwrap_or(&state.defaults.sampler))
                        .unwrap_or(state.defaults.sampler),
                })
            })
            .collect();

        // Step 4: Load materials
        let material_ids: Vec<_> = gltf
            .materials()
            .enumerate()
            .map(|(i, gltf_material)| {
                state.materials.insert(standard::material::load_material(
                    &gltf_material,
                    &texture_ids,
                    default_texture,
                    i,
                ))
            })
            .collect();

        // Step 5: Load meshes
        let accessors: Vec<_> = gltf.document.accessors().collect();
        let _mesh_ids: Vec<_> = gltf
            .meshes()
            .filter_map(|gltf_mesh| {
                // Add mesh from the GLTF to the state we're building
                let Ok((mesh, slot_materials_inv)) =
                    standard::mesh::load_mesh(&gltf_mesh, &accessors, &buffers, &material_ids)
                else {
                    return None;
                };
                let mesh_id = state.meshes.insert(mesh);
                state.mesh_materials.insert(mesh_id, slot_materials_inv);
                Some(mesh_id)
            })
            .collect();

        Ok(state)
    }
}
