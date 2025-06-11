use gltf::{self, Gltf};
use ordered_float::OrderedFloat;
use pp_core::{
    id::{self, Id},
    mesh::face::FaceDescriptor,
    State,
};
use std::{collections::HashMap, io::Cursor};

#[derive(Eq, Hash, PartialEq)]
struct VertPos([OrderedFloat<f32>; 3]);

impl From<[f32; 3]> for VertPos {
    fn from(value: [f32; 3]) -> Self {
        Self([value[0].into(), value[1].into(), value[2].into()])
    }
}

/// Creates a `State` object from a GLTF file. Note that this function merges
/// vertices with the same position
pub fn import_gltf() -> Result<State, gltf::Error> {
    let mut state = State::default();
    let cursor = Cursor::new(include_bytes!("./example/scene.glb"));
    let gltf = Gltf::from_reader(cursor)?;
    let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob)?;

    for mesh in gltf.document.meshes() {
        let i = pp_core::id::MeshId::from_usize(mesh.index());
        let mut pp_mesh = pp_core::mesh::Mesh::new(
            i,
            mesh.name().unwrap_or(format!("Mesh {i:?}").as_str()).into(),
        );

        let mut vertices: HashMap<VertPos, id::VertexId> = HashMap::new();
        for primitive in mesh.primitives() {
            let material = primitive.material();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            // Map GLTF vertices to mesh vertices by deduplicating on position
            let v_ids: Vec<_> = reader
                .read_positions()
                .unwrap()
                .map(|pos| *vertices.entry(pos.into()).or_insert_with(|| pp_mesh.add_vertex(pos)))
                .collect();

            // Now, create tris for each primitive. GLTF format has already ensured
            // that we're in the right type of primitive, e.g. no n-gons.
            let uvs: Option<Vec<_>> = reader.read_tex_coords(0).map(|uvs| uvs.into_f32().collect());
            let nos: Option<Vec<_>> = reader.read_normals().map(|nos| nos.collect());
            let indices: Vec<_> =
                reader.read_indices().unwrap().into_u32().map(|a| a as usize).collect();
            if primitive.mode() == gltf::mesh::Mode::Triangles {
                indices.chunks(3).for_each(|i| {
                    pp_mesh.add_face(
                        &[v_ids[i[0]], v_ids[i[1]], v_ids[i[2]]],
                        &FaceDescriptor {
                            uvs: uvs.as_ref().map(|uvs| [uvs[i[0]], uvs[i[1]], uvs[i[2]]]).as_ref(),
                            nos: nos.as_ref().map(|nos| [nos[i[0]], nos[i[1]], nos[i[2]]]).as_ref(),
                            ..Default::default()
                        },
                    );
                });
            };
        }

        // Add the mesh to the scene state
        state.meshes.insert(i, pp_mesh);
    }

    Ok(state)
}
