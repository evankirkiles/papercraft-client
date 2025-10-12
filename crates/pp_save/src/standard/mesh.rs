use pp_core::mesh::Mesh;
use slotmap::SecondaryMap;
use std::collections::{BTreeMap, HashMap};

use crate::standard::buffers::{AccessorOptions, GltfBufferBuilder};

/// Converts a pp_core::Mesh to GLTF primitives and adds them to the builder.
/// Note that one of our meshes might result in multiple GLTF meshes, because
/// GLTF meshes can only have a single material.
pub fn save_mesh(
    mesh: &Mesh,
    materials: &SecondaryMap<pp_core::mesh::MaterialSlotId, pp_core::MaterialId>,
    material_map: &HashMap<pp_core::MaterialId, gltf_json::Index<gltf_json::Material>>,
    builder: &mut GltfBufferBuilder,
) -> Vec<gltf_json::mesh::Primitive> {
    use gltf_json::validation::Checked::Valid;
    use gltf_json::{accessor, buffer, mesh};

    // Collect all loop data (positions, normals, UVs, material indices)
    let loops: Vec<_> = mesh.iter_loops().collect();

    // Build vertex data arrays
    let mut positions = Vec::with_capacity(loops.len());
    let mut normals = Vec::with_capacity(loops.len());
    let mut tex_coords = Vec::with_capacity(loops.len());
    let mut material_indices = Vec::with_capacity(loops.len());

    for loop_id in loops.iter() {
        let loop_ = &mesh[*loop_id];
        let vertex = &mesh[loop_.v];
        let face = &mesh[loop_.f];

        // Transform from OpenGL/WebGL coordinate system to GLTF coordinate system
        // GLTF uses right-handed Y-up, if your data is Z-up we need to rotate -90° around X
        // This transforms: X -> X, Y -> -Z, Z -> Y
        let pos = vertex.po;
        let transformed_pos = [pos[0], pos[2], -pos[1]];
        positions.push(transformed_pos);

        // Transform normals the same way
        let norm = loop_.no;
        let transformed_norm = [norm[0], norm[2], -norm[1]];
        normals.push(transformed_norm);

        // UV coordinates don't need transformation
        tex_coords.push(loop_.uv);

        // Map material slot to material index
        let mat_slot = face.m;
        material_indices.push(mat_slot);
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

    // Group triangles by material
    let mut primitives_by_material: HashMap<Option<pp_core::mesh::MaterialSlotId>, Vec<u32>> =
        HashMap::new();

    // Each loop is a triangle vertex, so we add indices in groups of 3
    for (i, mat_slot) in material_indices.iter().enumerate() {
        let indices = primitives_by_material.entry(*mat_slot).or_default();
        indices.push(i as u32);
    }

    // Create a primitive for each material
    let mut primitives = Vec::new();

    for (mat_slot, indices) in primitives_by_material {
        // Create indices accessor
        let indices_accessor = builder.add_accessor(
            &indices,
            AccessorOptions {
                component_type: Valid(accessor::GenericComponentType(accessor::ComponentType::U32)),
                type_: Valid(accessor::Type::Scalar),
                target: Some(Valid(buffer::Target::ElementArrayBuffer)),
                normalized: false,
                min: None,
                max: None,
            },
        );

        // Map material slot to material index
        let material_index = mat_slot
            .and_then(|slot| materials.get(slot))
            .and_then(|mat_id| material_map.get(mat_id))
            .copied();

        let primitive = mesh::Primitive {
            attributes: {
                let mut map = BTreeMap::new();
                map.insert(Valid(mesh::Semantic::Positions), position_accessor);
                map.insert(Valid(mesh::Semantic::Normals), normal_accessor);
                map.insert(Valid(mesh::Semantic::TexCoords(0)), texcoord_accessor);
                map
            },
            indices: Some(indices_accessor),
            material: material_index,
            mode: Valid(mesh::Mode::Triangles),
            targets: None,
            extensions: Default::default(),
            extras: Default::default(),
        };

        primitives.push(primitive);
    }

    primitives
}
