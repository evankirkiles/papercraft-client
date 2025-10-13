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
        let mut image_map = HashMap::new();
        let mut gltf_images = Vec::new();
        for (img_id, img) in self.images.iter() {
            if img_id == self.defaults.image {
                continue;
            }
            let gltf_image = standard::image::save_image(img);
            let idx = gltf_json::Index::new(gltf_images.len() as u32);
            gltf_images.push(gltf_image);
            image_map.insert(img_id, idx);
        }

        // Step 2: Save samplers (skip default sampler)
        let mut sampler_map = HashMap::new();
        let mut gltf_samplers = Vec::new();
        for (samp_id, samp) in self.samplers.iter() {
            if samp_id == self.defaults.sampler {
                continue;
            }
            let gltf_sampler = standard::sampler::save_sampler(samp);
            let idx = gltf_json::Index::new(gltf_samplers.len() as u32);
            gltf_samplers.push(gltf_sampler);
            sampler_map.insert(samp_id, idx);
        }

        // Step 3: Save textures (skip default texture)
        let mut texture_map = HashMap::new();
        let mut gltf_textures = Vec::new();
        for (tex_id, tex) in self.textures.iter() {
            if tex_id == self.defaults.texture {
                continue;
            }
            let gltf_texture = gltf_json::Texture {
                name: Some(tex.label.clone()),
                sampler: sampler_map.get(&tex.sampler).copied(),
                source: image_map.get(&tex.image).copied().unwrap(),
                extensions: Default::default(),
                extras: Default::default(),
            };
            let idx = gltf_json::Index::new(gltf_textures.len() as u32);
            gltf_textures.push(gltf_texture);
            texture_map.insert(tex_id, idx);
        }

        // Step 4: Save materials (skip default material)
        let mut material_map = HashMap::new();
        let mut gltf_materials = Vec::new();
        for (mat_id, mat) in self.materials.iter() {
            if mat_id == self.defaults.material {
                continue;
            }
            let gltf_material = standard::material::save_material(mat, &texture_map);
            let idx = gltf_json::Index::new(gltf_materials.len() as u32);
            gltf_materials.push(gltf_material);
            material_map.insert(mat_id, idx);
        }

        // Step 5: Save mesh geometries
        let mut mesh_map = HashMap::new();
        let mut gltf_meshes = Vec::new();
        for (mesh_id, mesh) in self.meshes.iter() {
            let materials = self.mesh_materials.get(mesh_id).cloned().unwrap_or_default();
            let gltf_mesh =
                standard::mesh::save_mesh(mesh, &materials, &material_map, &mut gltf_builder);
            let idx = gltf_json::Index::<u32>::new(gltf_meshes.len() as u32);
            gltf_meshes.push(gltf_mesh);
            mesh_map.insert(mesh_id, idx);
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
        Ok(SaveFile(Gltf {
            document: gltf::Document::from_json(gltf_json::Root {
                accessors,
                buffers,
                buffer_views,
                scene: Some(gltf_json::Index::new(0)),
                scenes: vec![gltf_json::Scene {
                    name: Some("Scene".to_string()),
                    nodes: (0..gltf_nodes.len() as u32).map(gltf_json::Index::new).collect(),
                    extensions: Default::default(),
                    extras: Default::default(),
                }],
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
