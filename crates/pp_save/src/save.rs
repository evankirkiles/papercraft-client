use gltf::Gltf;
use std::collections::HashMap;
use thiserror::Error;

use crate::{standard, SaveFile};

/// Possible errors that can occur while saving a file
#[derive(Debug, Clone, Copy, Error)]
pub enum SaveError {
    #[error("unknown data store error")]
    Unknown,
}

pub trait Saveable {
    fn save(&self) -> anyhow::Result<SaveFile>;
}

impl Saveable for pp_core::State {
    fn save(&self) -> anyhow::Result<SaveFile> {
        // We will be *building* a single large buffer with our data
        let mut gltf_builder = standard::buffers::GltfBufferBuilder::new();

        // Step 1: Save images (skip default image)
        let mut image_ids = HashMap::new();
        let mut images = Vec::new();
        let mut sorted_images: Vec<_> = self.images.iter().collect();
        sorted_images.sort_by_key(|(id, _)| *id);
        for (img_id, img) in sorted_images {
            if img_id == self.defaults.image {
                continue;
            }
            let gltf_image = standard::image::save_image(img);
            let idx = gltf_json::Index::new(images.len() as u32);
            images.push(gltf_image);
            image_ids.insert(img_id, idx);
        }

        // Step 2: Save samplers (skip default sampler)
        let mut sampler_ids = HashMap::new();
        let mut samplers = Vec::new();
        let mut sorted_samplers: Vec<_> = self.samplers.iter().collect();
        sorted_samplers.sort_by_key(|(id, _)| *id);
        for (samp_id, samp) in sorted_samplers {
            if samp_id == self.defaults.sampler {
                continue;
            }
            let gltf_sampler = standard::sampler::save_sampler(samp);
            let idx = gltf_json::Index::new(samplers.len() as u32);
            samplers.push(gltf_sampler);
            sampler_ids.insert(samp_id, idx);
        }

        // Step 3: Save textures (skip default texture)
        let mut texture_ids = HashMap::new();
        let mut textures = Vec::new();
        let mut sorted_textures: Vec<_> = self.textures.iter().collect();
        sorted_textures.sort_by_key(|(id, _)| *id);
        for (tex_id, tex) in sorted_textures {
            if tex_id == self.defaults.texture {
                continue;
            }
            let gltf_texture = gltf_json::Texture {
                name: Some(tex.label.clone()),
                sampler: sampler_ids.get(&tex.sampler).copied(),
                source: image_ids.get(&tex.image).copied().unwrap(),
                extensions: Default::default(),
                extras: Default::default(),
            };
            let idx = gltf_json::Index::new(textures.len() as u32);
            textures.push(gltf_texture);
            texture_ids.insert(tex_id, idx);
        }

        // Step 4: Save materials (skip default material)
        let mut material_ids = HashMap::new();
        let mut materials = Vec::new();
        let mut sorted_materials: Vec<_> = self.materials.iter().collect();
        sorted_materials.sort_by_key(|(id, _)| *id);
        for (mat_id, mat) in sorted_materials {
            if mat_id == self.defaults.material {
                continue;
            }
            let gltf_material = standard::material::save_material(mat, &texture_ids);
            let idx = gltf_json::Index::new(materials.len() as u32);
            materials.push(gltf_material);
            material_ids.insert(mat_id, idx);
        }

        // Step 5: Save mesh geometries
        let mut meshes = Vec::new();
        let mut sorted_meshes: Vec<_> = self.meshes.iter().collect();
        sorted_meshes.sort_by_key(|(id, _)| *id);
        for (_, mesh) in sorted_meshes {
            let gltf_mesh = standard::mesh::save_mesh(mesh, &material_ids, &mut gltf_builder);
            meshes.push(gltf_mesh);
        }

        // Step 6: Save each mesh as a node
        // In the future, we may support grouping / more ECS-style scene building
        let mut nodes = Vec::new();
        for i in 0u32..(self.meshes.len() as u32) {
            nodes.push(gltf_json::Node {
                name: Some(format!("Node_{}", i)),
                mesh: Some(gltf_json::Index::new(i)),
                ..Default::default()
            });
        }

        // Step 7: Create a scene from all the nodes
        // If there are no nodes, we need to set scene to None.
        let mut scene = None;
        let mut scenes = Vec::new();
        if !nodes.is_empty() {
            scene = Some(gltf_json::Index::new(0));
            scenes.push(gltf_json::Scene {
                name: Some("Scene".to_string()),
                nodes: (0..nodes.len() as u32).map(gltf_json::Index::new).collect(),
                extensions: Default::default(),
                extras: Default::default(),
            })
        }

        // Build final buffers, buffer views, and accessors
        let (buffers, buffer_views, accessors) = gltf_builder.build();
        Ok(SaveFile(Gltf {
            document: gltf::Document::from_json(gltf_json::Root {
                accessors,
                buffers,
                buffer_views,
                scene,
                scenes,
                meshes,
                nodes,
                samplers,
                textures,
                images,
                materials,
                ..Default::default()
            })?,
            blob: None,
        }))
    }
}
