use crate::{standard, SaveFile};
use gltf::Semantic;
use ordered_float::OrderedFloat;
use pp_core::{
    id::{self},
    material::texture::Texture,
    mesh::{face::FaceDescriptor, matslot::MaterialSlot, MaterialSlotId},
    MaterialId, State,
};
use slotmap::{SecondaryMap, SlotMap};
use std::collections::HashMap;
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

#[derive(Eq, Hash, PartialEq)]
struct VertPos([OrderedFloat<f32>; 3]);

impl From<[f32; 3]> for VertPos {
    fn from(value: [f32; 3]) -> Self {
        Self([value[0].into(), value[1].into(), value[2].into()])
    }
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
            .map(|(i, gltf_image)| state.images.insert(standard::image::load_image(&gltf_image, i)))
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
        for mesh in gltf.meshes() {
            let mut pp_mesh = pp_core::mesh::Mesh::new(
                mesh.name().map(|e| e.to_string()).unwrap_or_else(|| "ImportedMesh".to_string()),
            );
            let mut vertices: HashMap<VertPos, id::VertexId> = HashMap::new();
            let mut material_slots: SlotMap<MaterialSlotId, MaterialSlot> = SlotMap::with_key();
            let mut slot_materials: HashMap<MaterialId, MaterialSlotId> = HashMap::new();
            let mut slot_materials_inv: SecondaryMap<MaterialSlotId, MaterialId> =
                SecondaryMap::new();

            // Primitive corresponds to a set of faces with the same material
            for primitive in mesh.primitives() {
                use standard::buffers;
                let attrs: HashMap<_, _> = primitive.attributes().collect();

                // Read vertex attributes from the buffers
                let pos_acc = attrs.get(&Semantic::Positions).ok_or(LoadError::Unknown)?;
                let positions = buffers::read_accessor::<[f32; 3]>(&buffers, pos_acc)?;

                // Read indices - GLTF supports u8, u16, or u32 indices
                let ind_acc = primitive.indices().ok_or(LoadError::Unknown)?;
                let indices: Vec<u32> = match ind_acc.data_type() {
                    gltf::accessor::DataType::U8 => {
                        buffers::read_accessor::<u8>(&buffers, &ind_acc)?
                            .into_iter()
                            .map(|i| i as u32)
                            .collect()
                    }
                    gltf::accessor::DataType::U16 => {
                        buffers::read_accessor::<u16>(&buffers, &ind_acc)?
                            .into_iter()
                            .map(|i| i as u32)
                            .collect()
                    }
                    gltf::accessor::DataType::U32 => {
                        buffers::read_accessor::<u32>(&buffers, &ind_acc)?
                    }
                    _ => return Err(LoadError::Unknown),
                };

                // Normals and UVs are not strictly required like position / indices
                let normals = attrs.get(&Semantic::Normals).and_then(|accessor| {
                    buffers::read_accessor::<[f32; 3]>(&buffers, accessor).ok()
                });
                let uvs = attrs.get(&Semantic::TexCoords(0)).and_then(|accessor| {
                    buffers::read_accessor::<[f32; 2]>(&buffers, accessor).ok()
                });

                // Transform positions from GLTF (Y-up) back to internal (Z-up) coordinate system,
                // and deduplicate the vertices on their positions within the mesh itself. This
                // allows us to reconstruct adjacencies instead of treating all tris distinctly.
                // Note that we preserve the per-vertex normals / UVs by applying that data
                // to our BMesh `loops`.
                let v_ids: Vec<_> = positions
                    .iter()
                    .map(|pos| {
                        let transformed = [pos[0], -pos[2], pos[1]];
                        *vertices
                            .entry(transformed.into())
                            .or_insert_with(|| pp_mesh.add_vertex(transformed))
                    })
                    .collect();

                // Create a bidirectional map between material "slots" and "materials",
                // so we can re-use materials across meshes. Might remove this in the future.
                let material_slot = primitive.material().index().and_then(|mat_idx| {
                    material_ids.get(mat_idx).map(|m_id| {
                        *slot_materials.entry(*m_id).or_insert_with(|| {
                            let slot = material_slots.insert(MaterialSlot {
                                label: gltf
                                    .materials()
                                    .nth(mat_idx)
                                    .and_then(|e| e.name())
                                    .map(|e| e.to_string())
                                    .clone()
                                    .unwrap_or_default(),
                            });
                            slot_materials_inv.insert(slot, *m_id);
                            slot
                        })
                    })
                });

                // Create our adjacency map of tris from the indices
                for i in (0..indices.len()).step_by(3) {
                    let idx =
                        [indices[i] as usize, indices[i + 1] as usize, indices[i + 2] as usize];
                    pp_mesh.add_face(
                        &[v_ids[idx[0]], v_ids[idx[1]], v_ids[idx[2]]],
                        &FaceDescriptor {
                            m: material_slot,
                            uvs: uvs
                                .as_ref()
                                .map(|uvs| [uvs[idx[0]], uvs[idx[1]], uvs[idx[2]]])
                                .as_ref(),
                            nos: normals
                                .as_ref()
                                .map(|no| {
                                    [
                                        [no[idx[0]][0], -no[idx[0]][2], no[idx[0]][1]],
                                        [no[idx[1]][0], -no[idx[1]][2], no[idx[1]][1]],
                                        [no[idx[2]][0], -no[idx[2]][2], no[idx[2]][1]],
                                    ]
                                })
                                .as_ref(),
                        },
                    );
                }
            }

            // Add mesh from the GLTF to the state we're building
            let mesh_id = state.meshes.insert(pp_mesh);
            state.mesh_materials.insert(mesh_id, slot_materials_inv);
        }

        // Step 6: Load cuts and pieces from extras
        // TODO: Extract from root.extras when GLTF extension is implemented

        Ok(state)
    }
}
