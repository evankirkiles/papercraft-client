use gltf_json as json;
use gltf_json::material::{PbrBaseColorFactor, PbrMetallicRoughness};
use gltf_json::texture::Info;
use std::collections::HashMap;

/// Converts a pp_core Material to GLTF Material with texture references
pub fn save_material(
    mat: &pp_core::material::Material,
    texture_map: &HashMap<pp_core::TextureId, json::Index<json::Texture>>,
) -> json::Material {
    let base_color_texture = texture_map.get(&mat.base_color_texture).map(|&tex_idx| Info {
        index: tex_idx,
        tex_coord: 0,
        extensions: Default::default(),
        extras: Default::default(),
    });

    json::Material {
        name: Some(mat.label.clone()),
        pbr_metallic_roughness: PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor(mat.base_color_factor),
            base_color_texture,
            ..Default::default()
        },
        ..Default::default()
    }
}
