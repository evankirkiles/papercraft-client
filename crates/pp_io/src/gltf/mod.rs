use gltf::{self, Gltf};
use ordered_float::OrderedFloat;
use pp_core::{
    id::{self, Id},
    material::{
        image::{Format, Image},
        texture::{MinMagFilter, Sampler, Texture, WrappingMode},
        Material,
    },
    mesh::{face::FaceDescriptor, matslot::MaterialSlot, MaterialSlotId},
    MaterialId, State,
};
use slotmap::{SecondaryMap, SlotMap};
use std::{collections::HashMap, io::Cursor};

#[derive(Eq, Hash, PartialEq)]
struct VertPos([OrderedFloat<f32>; 3]);

impl From<[f32; 3]> for VertPos {
    fn from(value: [f32; 3]) -> Self {
        Self([value[0].into(), value[1].into(), value[2].into()])
    }
}

pub trait ImportGLTF {
    fn import_gltf(&mut self) -> Result<(), gltf::Error>;
}

impl ImportGLTF for State {
    /// Creates a `State` object from a GLTF file. Note that this function merges
    /// vertices with the same position
    fn import_gltf(&mut self) -> Result<(), gltf::Error> {
        let cursor = Cursor::new(include_bytes!("./example/Link.glb"));

        let gltf = Gltf::from_reader(cursor)?;
        let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob)?;
        let images = gltf::import_images(&gltf.document, None, &buffers)?;

        let image_ids: Vec<_> = gltf
            .document
            .images()
            .map(|image| {
                let i = pp_core::id::ImageId::from_usize(image.index());
                let image = &images[image.index()];
                // Map three-channel pixel data into four-channel pixel data (Alpha being 1)
                let pixels = match image.format {
                    gltf::image::Format::R8G8B8 => {
                        // Convert u8 RGB to u8 RGBA (alpha = 255)
                        image
                            .pixels
                            .chunks(3)
                            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                            .collect::<Vec<u8>>()
                    }
                    gltf::image::Format::R16G16B16 => {
                        // Convert u16 RGB to u16 RGBA (alpha = u16::MAX)
                        let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
                        for chunk in image.pixels.chunks_exact(6) {
                            let r = u16::from_le_bytes([chunk[0], chunk[1]]);
                            let g = u16::from_le_bytes([chunk[2], chunk[3]]);
                            let b = u16::from_le_bytes([chunk[4], chunk[5]]);
                            rgba.extend_from_slice(&r.to_le_bytes());
                            rgba.extend_from_slice(&g.to_le_bytes());
                            rgba.extend_from_slice(&b.to_le_bytes());
                            rgba.extend_from_slice(&u16::MAX.to_le_bytes());
                        }
                        rgba
                    }
                    gltf::image::Format::R32G32B32FLOAT => {
                        // Convert f32 RGB to f32 RGBA (alpha = 1.0)
                        let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
                        for chunk in image.pixels.chunks_exact(12) {
                            let r = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
                            let g = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
                            let b = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
                            rgba.extend_from_slice(&r.to_le_bytes());
                            rgba.extend_from_slice(&g.to_le_bytes());
                            rgba.extend_from_slice(&b.to_le_bytes());
                            rgba.extend_from_slice(&1.0f32.to_le_bytes());
                        }
                        rgba
                    }
                    _ => image.pixels.clone(),
                };
                self.images.insert(Image {
                    label: format!("{i:?}").as_str().into(),
                    pixels,
                    width: image.width,
                    height: image.height,
                    format: match image.format {
                        gltf::image::Format::R8 => Format::R8,
                        gltf::image::Format::R8G8 => Format::R8G8,
                        gltf::image::Format::R8G8B8A8 => Format::R8G8B8A8,
                        gltf::image::Format::R16 => Format::R16,
                        gltf::image::Format::R16G16 => Format::R16G16,
                        gltf::image::Format::R16G16B16A16 => Format::R16G16B16A16,
                        gltf::image::Format::R32G32B32A32FLOAT => Format::R32G32B32A32FLOAT,
                        // Three-channel textures are given an alpha channel so they can
                        // be handled by `wgpu` (which doesn't support 3-channel textures)
                        gltf::image::Format::R8G8B8 => Format::R8G8B8A8,
                        gltf::image::Format::R16G16B16 => Format::R16G16B16A16,
                        gltf::image::Format::R32G32B32FLOAT => Format::R32G32B32A32FLOAT,
                    },
                })
            })
            .collect();

        let default_sampler = self.defaults.sampler;
        let sampler_ids: Vec<_> = gltf
            .document
            .samplers()
            .map(|sampler| {
                self.samplers.insert(Sampler {
                    min_filter: sampler.min_filter().map(|min_filter| match min_filter {
                        gltf::texture::MinFilter::Nearest => MinMagFilter::Nearest,
                        gltf::texture::MinFilter::Linear => MinMagFilter::Linear,
                        _ => MinMagFilter::Nearest,
                    }),
                    mag_filter: sampler.mag_filter().map(|mag_filter| match mag_filter {
                        gltf::texture::MagFilter::Nearest => MinMagFilter::Nearest,
                        gltf::texture::MagFilter::Linear => MinMagFilter::Linear,
                    }),
                    wrap_u: match sampler.wrap_s() {
                        gltf::texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
                        gltf::texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
                        gltf::texture::WrappingMode::Repeat => WrappingMode::Repeat,
                    },
                    wrap_v: match sampler.wrap_t() {
                        gltf::texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
                        gltf::texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
                        gltf::texture::WrappingMode::Repeat => WrappingMode::Repeat,
                    },
                })
            })
            .collect();

        let default_texture = self.defaults.texture;
        let texture_ids: Vec<_> = gltf
            .document
            .textures()
            .map(|texture| {
                let i = pp_core::id::TextureId::from_usize(texture.index());
                self.textures.insert(Texture {
                    label: texture.name().unwrap_or(format!("{i:?}").as_str()).into(),
                    image: image_ids[texture.source().index()],
                    sampler: texture
                        .sampler()
                        .index()
                        .map(|s_id| sampler_ids[s_id])
                        .unwrap_or(default_sampler),
                })
            })
            .collect();

        let material_ids: Vec<_> = gltf
            .document
            .materials()
            .map(|material| {
                let i =
                    material.index().map(pp_core::id::MaterialId::from_usize).unwrap_or_default();
                let pbr_metallic_roughness = material.pbr_metallic_roughness();
                self.materials.insert(Material {
                    label: material.name().unwrap_or(format!("{i:?}").as_str()).into(),
                    base_color_texture: pbr_metallic_roughness
                        .base_color_texture()
                        .map(|info| texture_ids[info.texture().index()])
                        .unwrap_or(default_texture),
                    base_color_factor: pbr_metallic_roughness.base_color_factor(),
                    is_dirty: true,
                })
            })
            .collect();

        gltf.document.meshes().for_each(|mesh| {
            let i = pp_core::id::MeshId::from_usize(mesh.index());
            let mut pp_mesh = pp_core::mesh::Mesh::new(
                mesh.name().unwrap_or(format!("Mesh {i:?}").as_str()).into(),
            );
            // This map keeps track of all the materials used in the mesh
            let mut material_slots: SlotMap<MaterialSlotId, MaterialSlot> = SlotMap::with_key();
            let mut slot_materials: HashMap<MaterialId, MaterialSlotId> = HashMap::new();
            let mut slot_materials_inv: SecondaryMap<MaterialSlotId, MaterialId> =
                SecondaryMap::new();

            // Keep track of vertices keyed by their position, so we can preserve
            // topology despite GLTF splitting meshes up based on material
            let mut vertices: HashMap<VertPos, id::VertexId> = HashMap::new();
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                // Map GLTF vertices to mesh vertices by deduplicating on position
                let v_ids: Vec<_> = reader
                    .read_positions()
                    .unwrap()
                    .map(|pos| {
                        *vertices.entry(pos.into()).or_insert_with(|| pp_mesh.add_vertex(pos))
                    })
                    .collect();

                // Map the tri's global material to a proper slot within the mesh
                let material = primitive.material();
                let material_slot = material
                    .index()
                    .map(|m_id| material_ids.get(m_id).unwrap())
                    .map(|m_id| {
                        // First check if we already have a slot for this material
                        slot_materials.entry(*m_id).or_insert_with(|| {
                            // If not, create a mesh slot for this material
                            let slot = material_slots.insert(MaterialSlot {
                                label: material.name().unwrap_or("").to_string(),
                            });
                            slot_materials_inv.insert(slot, *m_id);
                            slot
                        })
                    })
                    .cloned();

                // Now, create tris for each primitive. The GLTF format has already ensured
                // that we're in the right type of primitive, only tris, no n-gons or points or lines.
                let uvs: Option<Vec<_>> =
                    reader.read_tex_coords(0).map(|uvs| uvs.into_f32().collect());
                let nos: Option<Vec<_>> = reader.read_normals().map(|nos| nos.collect());
                let indices: Vec<_> =
                    reader.read_indices().unwrap().into_u32().map(|a| a as usize).collect();
                if primitive.mode() == gltf::mesh::Mode::Triangles {
                    indices.chunks(3).for_each(|i| {
                        pp_mesh.add_face(
                            &[v_ids[i[0]], v_ids[i[1]], v_ids[i[2]]],
                            &FaceDescriptor {
                                uvs: uvs
                                    .as_ref()
                                    .map(|uvs| [uvs[i[0]], uvs[i[1]], uvs[i[2]]])
                                    .as_ref(),
                                nos: nos
                                    .as_ref()
                                    .map(|nos| [nos[i[0]], nos[i[1]], nos[i[2]]])
                                    .as_ref(),
                                m: material_slot,
                            },
                        );
                    });
                };
            }

            // Add the mesh to the scene state
            self.add_mesh(pp_mesh, Some(slot_materials_inv));
        });

        Ok(())
    }
}
