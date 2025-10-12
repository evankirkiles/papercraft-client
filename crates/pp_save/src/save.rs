use std::collections::HashMap;

use crate::{gltf, SaveFile};

/// Possible errors that can occur while saving a file
#[derive(Debug, Clone, Copy)]
pub enum SaveError {
    Unknown,
}

pub trait Saveable {
    fn save(&mut self) -> Result<SaveFile, SaveError>;
}

impl Saveable for pp_core::State {
    fn save(&mut self) -> Result<SaveFile, SaveError> {
        let mut gltf_builder = gltf::buffers::GltfBufferBuilder::new();

        // Section 1: GLTF Geometry

        // Step 1: Save images (even pixel data, for now)
        let mut image_map = HashMap::new();
        let mut gltf_images = Vec::new();
        for (img_id, img) in self.images.iter() {
            let gltf_image = gltf::image::save_image(img);
            let idx = gltf_json::Index::new(gltf_images.len() as u32);
            gltf_images.push(gltf_image);
            image_map.insert(img_id, idx);
        }

        // Step 2: Save samplers
        let mut sampler_map = HashMap::new();
        let mut gltf_samplers = Vec::new();
        for (samp_id, samp) in self.samplers.iter() {
            let gltf_sampler = gltf::image::save_sampler(samp);
            let idx = gltf_json::Index::new(gltf_samplers.len() as u32);
            gltf_samplers.push(gltf_sampler);
            sampler_map.insert(samp_id, idx);
        }

        // Step 3: Save textures
        let mut texture_map = HashMap::new();
        let mut gltf_textures = Vec::new();
        for (tex_id, tex) in self.textures.iter() {
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

        // Step 4: Save materials
        let mut material_map = HashMap::new();
        let mut gltf_materials = Vec::new();
        for (mat_id, mat) in self.materials.iter() {
            let gltf_material = gltf::material::save_material(mat, &texture_map);
            let idx = gltf_json::Index::new(gltf_materials.len() as u32);
            gltf_materials.push(gltf_material);
            material_map.insert(mat_id, idx);
        }

        // Step 5: Save mesh geometries
        let mut gltf_meshes = Vec::new();
        for (mesh_id, mesh) in self.meshes.iter() {
            let materials = self.mesh_materials.get(mesh_id).cloned().unwrap_or_default();
            let primitives =
                gltf::mesh::save_mesh(mesh, &materials, &material_map, &mut gltf_builder);
            gltf_meshes.push(gltf_json::Mesh {
                name: Some(mesh.label.clone()),
                primitives,
                weights: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
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

        // Section 2: GLTF Papercraft Extra

        // Step 1: Save cuts

        // Step 2: Save pieces

        // Build final buffers, buffer views, and accessors
        let (buffers, buffer_views, accessors) = gltf_builder.build();
        Ok(SaveFile(gltf_json::Root {
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
            // extras: Some(Box::new(3)),
            meshes: gltf_meshes,
            nodes: gltf_nodes,
            samplers: gltf_samplers,
            textures: gltf_textures,
            images: gltf_images,
            materials: gltf_materials,
            ..Default::default()
        }))
    }
}
