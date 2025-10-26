use gltf::{buffer::Data, Accessor, Semantic};
use ordered_float::OrderedFloat;
use pp_core::{
    id::{self, FaceId, VertexId},
    mesh::{face::FaceDescriptor, Mesh},
    MaterialId,
};
use serde_json::value::RawValue;
use std::collections::{BTreeMap, HashMap};

use crate::{
    extra::{
        self, cut::SerializableCut, piece::SerializablePiece, MeshExtras, PapercraftMeshExtra,
    },
    load::LoadError,
    standard::buffers::{AccessorOptions, GltfBufferBuilder},
};

/// Converts a pp_core::Mesh to GLTF primitives and adds them to the builder.
/// Note that one of our meshes might result in multiple GLTF meshes, because
/// GLTF meshes can only have a single material.
pub fn save_mesh(
    mesh: &Mesh,
    materials: &HashMap<pp_core::MaterialId, gltf_json::Index<gltf_json::Material>>,
    builder: &mut GltfBufferBuilder,
) -> gltf_json::mesh::Mesh {
    use gltf_json::validation::Checked::Valid;
    use gltf_json::{accessor, buffer, mesh};

    // Collect all loop data (positions, normals, UVs, material indices)
    let loops: Vec<_> = mesh.iter_loops().collect();

    // Build vertex data arrays
    let mut positions = Vec::with_capacity(loops.len());
    let mut normals = Vec::with_capacity(loops.len());
    let mut tex_coords = Vec::with_capacity(loops.len());
    // Primitives are grouped by material
    let mut primitives_by_material: BTreeMap<Option<MaterialId>, Vec<u32>> = BTreeMap::new();

    // The below lookup table allows us to get (one of) a vertex's GLTF buffer
    // indices by its vertex ID, necessary for identifying the verts in an edge
    let mut v_indices: HashMap<VertexId, u32> = HashMap::new();
    let mut f_indices: HashMap<FaceId, u32> = HashMap::new();

    // GLTFs are just triangles, a lot of the time - meaning we should take a
    // per-face per-vertex approach to building our GLTF.
    for (loop_id, i) in loops.iter().zip(0u32..) {
        let loop_ = &mesh[*loop_id];
        let vertex = &mesh[loop_.v];
        let face = &mesh[loop_.f];

        // Transform from OpenGL/WebGL coordinate system to GLTF coordinate system
        // Obviously, UV coordinates don't need transformation
        positions.push([vertex.po[0], vertex.po[2], -vertex.po[1]]);
        normals.push([loop_.no[0], loop_.no[2], -loop_.no[1]]);
        tex_coords.push(loop_.uv);
        primitives_by_material.entry(face.m).or_default().push(i);

        // Store the to-be indices of the current vert and face. We'll use these
        // to consistently identify edges and pieces in the GLTF.
        v_indices.entry(loop_.v).or_insert(i);
        f_indices.entry(loop_.f).or_insert(i);
    }

    // Calculate min/max bounds for positions (required by GLTF spec for position accessors)
    let (min_pos, max_pos) = if positions.is_empty() {
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
    } else {
        let mut min = positions[0];
        let mut max = positions[0];
        for pos in &positions {
            min[0] = min[0].min(pos[0]);
            min[1] = min[1].min(pos[1]);
            min[2] = min[2].min(pos[2]);
            max[0] = max[0].max(pos[0]);
            max[1] = max[1].max(pos[1]);
            max[2] = max[2].max(pos[2]);
        }
        (min, max)
    };

    // Create accessors for vertex data
    let position_accessor = builder.add_accessor(
        &positions,
        AccessorOptions {
            component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
            type_: Valid(accessor::Type::Vec3),
            target: Some(Valid(buffer::Target::ArrayBuffer)),
            normalized: false,
            min: Some(gltf_json::Value::from(Vec::from(min_pos))),
            max: Some(gltf_json::Value::from(Vec::from(max_pos))),
        },
    );

    let normal_accessor = builder.add_accessor(
        &normals,
        AccessorOptions {
            component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
            type_: Valid(accessor::Type::Vec3),
            target: Some(Valid(buffer::Target::ArrayBuffer)),
            normalized: false,
            min: None,
            max: None,
        },
    );

    let texcoord_accessor = builder.add_accessor(
        &tex_coords,
        AccessorOptions {
            component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::F32)),
            type_: Valid(accessor::Type::Vec2),
            target: Some(Valid(buffer::Target::ArrayBuffer)),
            normalized: false,
            min: None,
            max: None,
        },
    );

    // Create a primitive for each material
    // BTreeMap iterates in sorted order, ensuring deterministic ordering
    let primitives = primitives_by_material
        .iter()
        .map(|(mat_id, indices)| mesh::Primitive {
            indices: Some(builder.add_accessor(
                indices,
                AccessorOptions {
                    component_type: Valid(accessor::GenericComponentType(
                        accessor::ComponentType::U32,
                    )),
                    type_: Valid(accessor::Type::Scalar),
                    target: Some(Valid(buffer::Target::ElementArrayBuffer)),
                    normalized: false,
                    min: None,
                    max: None,
                },
            )),
            attributes: {
                let mut map = BTreeMap::new();
                map.insert(Valid(mesh::Semantic::Positions), position_accessor);
                map.insert(Valid(mesh::Semantic::Normals), normal_accessor);
                map.insert(Valid(mesh::Semantic::TexCoords(0)), texcoord_accessor);
                map
            },
            material: mat_id.as_ref().and_then(|mat_id| materials.get(mat_id)).copied(),
            mode: Valid(mesh::Mode::Triangles),
            targets: None,
            extensions: Default::default(),
            extras: Default::default(),
        })
        .collect();

    // In our `extras` property, we need to indicate all the cut edges in this
    // mesh. However, edges are not a concept in GLTF-land. We instead identify
    // edges with pairs of vertices.
    let cuts = extra::cut::save_cuts(
        builder,
        mesh.cuts
            .iter()
            .filter(|(_, cut)| !cut.is_dead)
            .map(|(e_id, cut)| SerializableCut {
                vertices: [
                    *v_indices.get(&mesh[*e_id].v[0]).unwrap(),
                    *v_indices.get(&mesh[*e_id].v[1]).unwrap(),
                ],
                flap_position: cut.flap_position,
            })
            .collect(),
    );

    // In our `extras` property, we also need to include each of the pieces,
    // as each piece has metadata attached to it.
    let pieces = extra::piece::save_pieces(
        builder,
        mesh.iter_pieces()
            .map(|f_id| SerializablePiece {
                face_index: *f_indices.get(f_id).unwrap(),
                transform: mesh.pieces.get(f_id).unwrap().transform,
            })
            .collect(),
    );

    gltf_json::mesh::Mesh {
        name: mesh.label.clone(),
        primitives,
        weights: Default::default(),
        extensions: Default::default(),
        extras: serde_json::to_string(&MeshExtras {
            papercraft: Some(PapercraftMeshExtra { cuts, pieces }),
        })
        .ok()
        .and_then(|str| RawValue::from_string(str).ok()),
    }
}

#[derive(Eq, Hash, PartialEq)]
struct VertPos([OrderedFloat<f32>; 3]);

impl From<[f32; 3]> for VertPos {
    fn from(value: [f32; 3]) -> Self {
        Self([value[0].into(), value[1].into(), value[2].into()])
    }
}

/// Loads a mesh into runtime memory from its GLTF representation.
pub fn load_mesh(
    mesh: &gltf::mesh::Mesh,
    accessors: &[Accessor],
    buffers: &[Data],
    materials: &[MaterialId],
) -> anyhow::Result<pp_core::mesh::Mesh, LoadError> {
    let mut pp_mesh = pp_core::mesh::Mesh::new(
        mesh.name().map(|e| e.to_string()).unwrap_or_else(|| "ImportedMesh".to_string()),
    );
    let mut vertices: HashMap<VertPos, id::VertexId> = HashMap::new();

    // Build mappings from GLTF buffer indices to our runtime IDs
    // Key: GLTF buffer index, Value: our vertex/face ID
    let mut gltf_index_to_vertex_id: HashMap<u32, id::VertexId> = HashMap::new();
    let mut gltf_index_to_face_id: HashMap<u32, id::FaceId> = HashMap::new();

    // Primitive corresponds to a set of faces with the same material
    for primitive in mesh.primitives() {
        use crate::standard::buffers;
        let attrs: HashMap<_, _> = primitive.attributes().collect();

        // Read vertex attributes from the buffers
        let pos_acc = attrs.get(&Semantic::Positions).ok_or(LoadError::Unknown)?;
        let positions = buffers::read_accessor::<[f32; 3]>(buffers, pos_acc)?;

        // Read indices - GLTF supports u8, u16, or u32 indices
        let ind_acc = primitive.indices().ok_or(LoadError::Unknown)?;
        let indices: Vec<u32> = match ind_acc.data_type() {
            gltf::accessor::DataType::U8 => buffers::read_accessor::<u8>(buffers, &ind_acc)?
                .into_iter()
                .map(|i| i as u32)
                .collect(),
            gltf::accessor::DataType::U16 => buffers::read_accessor::<u16>(buffers, &ind_acc)?
                .into_iter()
                .map(|i| i as u32)
                .collect(),
            gltf::accessor::DataType::U32 => buffers::read_accessor::<u32>(buffers, &ind_acc)?,
            _ => return Err(LoadError::Unknown),
        };

        // Normals and UVs are not strictly required like position / indices
        let normals = attrs
            .get(&Semantic::Normals)
            .and_then(|accessor| buffers::read_accessor::<[f32; 3]>(buffers, accessor).ok());
        let uvs = attrs
            .get(&Semantic::TexCoords(0))
            .and_then(|accessor| buffers::read_accessor::<[f32; 2]>(buffers, accessor).ok());

        // Transform positions from GLTF (Y-up) back to internal (Z-up) coordinate system,
        // and deduplicate the vertices on their positions within the mesh itself. This
        // allows us to reconstruct adjacencies instead of treating all tris distinctly.
        // Note that we preserve the per-vertex normals / UVs by applying that data
        // to our BMesh `loops`.
        let v_ids: Vec<_> = positions
            .iter()
            .enumerate()
            .map(|(gltf_idx, pos)| {
                let transformed = [pos[0], -pos[2], pos[1]];
                let v_id = *vertices
                    .entry(transformed.into())
                    .or_insert_with(|| pp_mesh.add_vertex(transformed));

                // Store mapping from GLTF buffer index to vertex ID
                gltf_index_to_vertex_id.insert(gltf_idx as u32, v_id);
                v_id
            })
            .collect();

        // Create our adjacency map of tris from the indices by adding primitives
        // as faces.
        for i in (0..indices.len()).step_by(3) {
            let idx = [indices[i] as usize, indices[i + 1] as usize, indices[i + 2] as usize];
            let f_id = pp_mesh.add_face(
                &[v_ids[idx[0]], v_ids[idx[1]], v_ids[idx[2]]],
                &FaceDescriptor {
                    m: primitive
                        .material()
                        .index()
                        .and_then(|mat_idx| materials.get(mat_idx).cloned()),
                    uvs: uvs.as_ref().map(|uvs| [uvs[idx[0]], uvs[idx[1]], uvs[idx[2]]]).as_ref(),
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

            // Store mapping from GLTF buffer index to face ID
            // Use the first vertex's index as the face's index
            gltf_index_to_face_id.insert(indices[i], f_id);
        }
    }

    // Now we have our base geometry loaded in. It's time to load in all the cuts
    // and pieces contained in the mesh.
    let Some(extras) = mesh
        .extras()
        .clone()
        .and_then(|extras| serde_json::from_str::<crate::extra::MeshExtras>(extras.get()).ok())
        .and_then(|extras| extras.papercraft)
    else {
        return Ok(pp_mesh);
    };

    // 1. Load cuts and apply them to real edges in the model. Do *not* use our
    // internal functions which also create pieces / edges - we'll do that manually.
    crate::extra::cut::load_cuts(accessors, buffers, extras.cuts).iter().for_each(|cut| {
        if let (Some(&v0_id), Some(&v1_id)) = (
            gltf_index_to_vertex_id.get(&cut.vertices[0]),
            gltf_index_to_vertex_id.get(&cut.vertices[1]),
        ) {
            if let Some(e_id) = pp_mesh.query_edge(v0_id, v1_id) {
                pp_mesh.make_cut(e_id, false);
                if let Some(new_cut) = pp_mesh.cuts.get_mut(&e_id) {
                    new_cut.flap_position = cut.flap_position;
                }
            };
        };
    });

    // 2. Load pieces based on face IDs - we need to be able to consistently
    // refer to pieces such that we can load in their transforms / metadata.
    crate::extra::piece::load_pieces(accessors, buffers, extras.pieces).iter().for_each(
        |piece_data| {
            if let Some(f_id) = gltf_index_to_face_id.get(&piece_data.face_index) {
                pp_mesh.expand_piece(*f_id).unwrap();
                if let Some(piece) = pp_mesh.pieces.get_mut(f_id) {
                    piece.transform = piece_data.transform;
                    piece.elem_dirty = true;
                }
            };
        },
    );

    Ok(pp_mesh)
}
