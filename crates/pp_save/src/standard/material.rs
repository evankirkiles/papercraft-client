use pp_core::material::Material;
use pp_core::TextureId;
use std::collections::HashMap;

/// Converts a pp_core Material to GLTF Material with texture references
pub fn save_material(
    mat: &pp_core::material::Material,
    texture_map: &HashMap<pp_core::TextureId, gltf_json::Index<gltf_json::Texture>>,
) -> gltf_json::Material {
    use gltf_json::{material, texture};
    material::Material {
        name: Some(mat.label.clone()),
        pbr_metallic_roughness: material::PbrMetallicRoughness {
            base_color_factor: material::PbrBaseColorFactor(mat.base_color_factor),
            base_color_texture: texture_map.get(&mat.base_color_texture).map(|&tex_idx| {
                texture::Info {
                    index: tex_idx,
                    tex_coord: 0,
                    extensions: Default::default(),
                    extras: Default::default(),
                }
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn load_material(
    gltf_material: &gltf::Material,
    texture_ids: &[TextureId],
    default_texture: TextureId,
    index: usize,
) -> Material {
    let pbr = &gltf_material.pbr_metallic_roughness();
    Material {
        label: gltf_material
            .name()
            .map(|e| e.to_string())
            .unwrap_or_else(|| format!("Material{}", index)),
        base_color_texture: pbr
            .base_color_texture()
            .and_then(|info| texture_ids.get(info.texture().index()))
            .copied()
            .unwrap_or(default_texture),
        base_color_factor: pbr.base_color_factor(),
        is_dirty: true,
    }
}
