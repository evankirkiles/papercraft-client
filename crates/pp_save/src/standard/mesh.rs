use pp_core::{
    id::{FaceId, VertexId},
    mesh::Mesh,
};
use serde_json::value::RawValue;
use slotmap::SecondaryMap;
use std::collections::{BTreeMap, HashMap};

use crate::{
    extra::{
        self, cut::SerializableCut, piece::SerializablePiece, MeshExtras, PapercraftMeshExtra,
    },
    standard::buffers::{AccessorOptions, GltfBufferBuilder},
};

/// Converts a pp_core::Mesh to GLTF primitives and adds them to the builder.
/// Note that one of our meshes might result in multiple GLTF meshes, because
/// GLTF meshes can only have a single material.
pub fn save_mesh(
    mesh: &Mesh,
    materials: &SecondaryMap<pp_core::mesh::MaterialSlotId, pp_core::MaterialId>,
    material_map: &HashMap<pp_core::MaterialId, gltf_json::Index<gltf_json::Material>>,
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
    let mut primitives_by_material: HashMap<Option<pp_core::mesh::MaterialSlotId>, Vec<u32>> =
        HashMap::new();

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
    let primitives = primitives_by_material
        .iter()
        .map(|(mat_slot, indices)| mesh::Primitive {
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
            material: mat_slot
                .and_then(|slot| materials.get(slot))
                .and_then(|mat_id| material_map.get(mat_id))
                .copied(),
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
        mesh.edges
            .iter()
            .filter(|(_, e)| e.cut.is_some())
            .map(|(_, e)| SerializableCut {
                vertices: [*v_indices.get(&e.v[0]).unwrap(), *v_indices.get(&e.v[1]).unwrap()],
                flap_position: extra::cut::FlapPosition::FirstFace,
            })
            .collect(),
    );

    // In our `extras` property, we also need to include each of the pieces,
    // as each piece has metadata attached to it.
    let pieces = extra::piece::save_pieces(
        builder,
        mesh.pieces
            .iter()
            .map(|(_, piece)| SerializablePiece {
                face_index: *f_indices.get(&piece.f).unwrap(),
                transform: piece.transform,
            })
            .collect(),
    );

    gltf_json::mesh::Mesh {
        name: Some(mesh.label.clone()),
        primitives,
        weights: None,
        extensions: Default::default(),
        extras: serde_json::to_string(&MeshExtras {
            papercraft: Some(PapercraftMeshExtra { cuts, pieces }),
        })
        .ok()
        .and_then(|str| RawValue::from_string(str).ok()),
    }
}
