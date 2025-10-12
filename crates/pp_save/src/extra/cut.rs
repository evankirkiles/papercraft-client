use pp_core::{id::Id, State};
use serde::{Deserialize, Serialize};

/// Represents a cut edge in the mesh
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cut {
    /// The mesh this cut belongs to
    pub mesh: u32,
    /// The indices of the two vertices that form the cut edge
    pub vertices: [u32; 2],
    /// Whether this cut has a flap attached
    pub has_flap: bool,
}

/// Converts cut edges from pp_core State to PPR format
pub fn save_cuts(state: &State) -> Vec<Cut> {
    let mut cuts = Vec::new();
    for (mesh_idx, (_mesh_id, mesh)) in state.meshes.iter().enumerate() {
        for edge_id in mesh.edges.indices() {
            let edge_id = pp_core::id::EdgeId::from_usize(edge_id);
            let edge = &mesh[edge_id];
            if edge.cut.is_some() {
                cuts.push(Cut {
                    mesh: mesh_idx as u32,
                    vertices: [edge.v[0].to_usize() as u32, edge.v[1].to_usize() as u32],
                    has_flap: edge.cut.unwrap().l_flap.is_some(),
                });
            }
        }
    }
    cuts
}

/// Converts cut edges from pp_core State to PPR format
pub fn load_cuts(state: &mut State, cuts: &Vec<Cut>) {
    todo!()
}
