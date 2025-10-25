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
        let mut images = HashMap::new();
        let mut gltf_images = Vec::new();
        for (img_id, img) in self.images.iter() {
            if img_id == self.defaults.image {
                continue;
            }
            let gltf_image = standard::image::save_image(img);
            let idx = gltf_json::Index::new(gltf_images.len() as u32);
            gltf_images.push(gltf_image);
            images.insert(img_id, idx);
        }

        // Step 2: Save samplers (skip default sampler)
        let mut samplers = HashMap::new();
        let mut gltf_samplers = Vec::new();
        for (samp_id, samp) in self.samplers.iter() {
            if samp_id == self.defaults.sampler {
                continue;
            }
            let gltf_sampler = standard::sampler::save_sampler(samp);
            let idx = gltf_json::Index::new(gltf_samplers.len() as u32);
            gltf_samplers.push(gltf_sampler);
            samplers.insert(samp_id, idx);
        }

        // Step 3: Save textures (skip default texture)
        let mut textures = HashMap::new();
        let mut gltf_textures = Vec::new();
        for (tex_id, tex) in self.textures.iter() {
            if tex_id == self.defaults.texture {
                continue;
            }
            let gltf_texture = gltf_json::Texture {
                name: Some(tex.label.clone()),
                sampler: samplers.get(&tex.sampler).copied(),
                source: images.get(&tex.image).copied().unwrap(),
                extensions: Default::default(),
                extras: Default::default(),
            };
            let idx = gltf_json::Index::new(gltf_textures.len() as u32);
            gltf_textures.push(gltf_texture);
            textures.insert(tex_id, idx);
        }

        // Step 4: Save materials (skip default material)
        let mut materials = HashMap::new();
        let mut gltf_materials = Vec::new();
        for (mat_id, mat) in self.materials.iter() {
            if mat_id == self.defaults.material {
                continue;
            }
            let gltf_material = standard::material::save_material(mat, &textures);
            let idx = gltf_json::Index::new(gltf_materials.len() as u32);
            gltf_materials.push(gltf_material);
            materials.insert(mat_id, idx);
        }

        // Step 5: Save mesh geometries
        let mut gltf_meshes = Vec::new();
        for mesh in self.meshes.values() {
            let gltf_mesh = standard::mesh::save_mesh(mesh, &materials, &mut gltf_builder);
            gltf_meshes.push(gltf_mesh);
        }

        // Step 6: Create a scene with all meshes as nodes
        let mut gltf_nodes = Vec::new();
        for (i, _) in self.meshes.iter().enumerate() {
            gltf_nodes.push(gltf_json::Node {
                name: Some(format!("Node_{}", i)),
                mesh: Some(gltf_json::Index::new(i as u32)),
                camera: None,
                children: None,
                extensions: Default::default(),
                extras: Default::default(),
                matrix: None,
                rotation: None,
                scale: None,
                translation: None,
                skin: None,
                weights: None,
            });
        }

        // Build final buffers, buffer views, and accessors
        let (buffers, buffer_views, accessors) = gltf_builder.build();

        // If there are no nodes, we need to set scene to None
        // glTF spec allows scenes to reference nodes, but empty scenes are valid
        let (scene_index, scenes) = if gltf_nodes.is_empty() {
            // No nodes means no scene needed
            (None, vec![])
        } else {
            (
                Some(gltf_json::Index::new(0)),
                vec![gltf_json::Scene {
                    name: Some("Scene".to_string()),
                    nodes: (0..gltf_nodes.len() as u32).map(gltf_json::Index::new).collect(),
                    extensions: Default::default(),
                    extras: Default::default(),
                }],
            )
        };

        Ok(SaveFile(Gltf {
            document: gltf::Document::from_json(gltf_json::Root {
                accessors,
                buffers,
                buffer_views,
                scene: scene_index,
                scenes,
                meshes: gltf_meshes,
                nodes: gltf_nodes,
                samplers: gltf_samplers,
                textures: gltf_textures,
                images: gltf_images,
                materials: gltf_materials,
                ..Default::default()
            })?,
            blob: None,
        }))
    }
}
