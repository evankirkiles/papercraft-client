use std::io::Cursor;

use pp_core::{id::Id, mesh::face::FaceDescriptor, State};

/// Creates a `State` object from a Wavefront OBJ file.
pub fn import_obj() -> State {
    let mut state = State::default();
    let cursor = Cursor::new(include_bytes!("./example/penguin/PenguinBaseMesh.obj"));
    let model = tobj::load_obj_buf(
        &mut cursor.clone(),
        &tobj::LoadOptions {
            single_index: false,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        },
        |_| {
            let cursor = Cursor::new(include_bytes!("./example/penguin/PenguinBaseMesh.mtl"));
            tobj::load_mtl_buf(&mut cursor.clone())
        },
    );
    let (models, materials) = model.expect("Failed to load OBJ file");
    let materials = materials.expect("Failed to load MTL file");

    // Create each material
    // for (i, m) in materials.iter().enumerate() {
    //     let mut material = pp_core::material::Material::new();
    // }

    // Create each mesh
    for (i, m) in models.iter().enumerate() {
        let mut mesh = pp_core::mesh::Mesh::new(pp_core::id::MeshId::from_usize(i), m.name.clone());
        let mesh_data = &m.mesh;
        let mut vertex_map = Vec::with_capacity(mesh_data.positions.len() / 3);
        for i in 0..(mesh_data.positions.len() / 3) {
            let pos = [
                mesh_data.positions[3 * i],
                mesh_data.positions[3 * i + 1],
                mesh_data.positions[3 * i + 2],
            ];

            let normal = if !mesh_data.normals.is_empty() {
                [
                    mesh_data.normals[3 * i],
                    mesh_data.normals[3 * i + 1],
                    mesh_data.normals[3 * i + 2],
                ]
            } else {
                [0.0, 0.0, 0.0]
            };

            let vid = mesh.add_vertex(pos);
            vertex_map.push(vid);
        }

        // Faces are triangles because we asked for triangulation
        let has_uvs = !mesh_data.texcoords.is_empty() && !mesh_data.texcoord_indices.is_empty();
        for i in 0..(mesh_data.indices.len() / 3) {
            let idx0 = mesh_data.indices[3 * i] as usize;
            let idx1 = mesh_data.indices[3 * i + 1] as usize;
            let idx2 = mesh_data.indices[3 * i + 2] as usize;
            let face = [vertex_map[idx0], vertex_map[idx1], vertex_map[idx2]];
            let uv_data = has_uvs.then(|| {
                let uv_idx = |j| mesh_data.texcoord_indices[3 * i + j] as usize;
                [
                    [mesh_data.texcoords[2 * uv_idx(0)], mesh_data.texcoords[2 * uv_idx(0) + 1]],
                    [mesh_data.texcoords[2 * uv_idx(1)], mesh_data.texcoords[2 * uv_idx(1) + 1]],
                    [mesh_data.texcoords[2 * uv_idx(2)], mesh_data.texcoords[2 * uv_idx(2) + 1]],
                ]
            });
            mesh.add_face(&face, &FaceDescriptor { uvs: uv_data.as_ref(), ..Default::default() });
        }
        state.meshes.insert(mesh.id, mesh);
    }

    state
}
