use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    id::{self},
    MeshId,
};

use super::{Command, CommandError};

/// Cuts & joins edges, creating any resulting pieces from the operation. On each
/// cut, we save a before / after of any pieces on either side of each edge, as
/// well as a snapshot of any pieces involved in the operation (before OR after,
/// which is fine because no piece-internal data is changed as a result of cuts).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MakeCutsCommand {
    pub edges: Vec<(MeshId, id::EdgeId)>,
}

impl MakeCutsCommand {
    pub fn from_select(state: &mut crate::State) -> Self {
        let cmd = Self {
            edges: state
                .selection
                .edges
                .iter()
                .filter(|(m_id, e_id)| {
                    let mesh = &state.meshes[*m_id];
                    mesh.cuts.get(e_id).is_none_or(|cut| cut.is_dead)
                        && mesh.iter_edge_loops(*e_id).is_some_and(|mut walker| {
                            !walker.all(|l| state.selection.faces.contains(&(*m_id, mesh[l].f)))
                        })
                })
                .copied()
                .collect(),
        };
        cmd.execute(state).unwrap();
        cmd
    }
}

impl Command for MakeCutsCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        // Group edges by mesh
        let mut edges_by_mesh: HashMap<MeshId, Vec<id::EdgeId>> = HashMap::new();
        self.edges.iter().for_each(|(mesh_id, edge_id)| {
            edges_by_mesh.entry(*mesh_id).or_default().push(*edge_id);
        });

        // Make the cuts for each mesh in forward order
        edges_by_mesh.iter().for_each(|(mesh_id, edge_ids)| {
            if let Some(mesh) = state.meshes.get_mut(*mesh_id) {
                edge_ids.iter().for_each(|e_id| mesh.make_cut(*e_id, true));
            }
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        // Group edges by mesh
        let mut edges_by_mesh: HashMap<MeshId, Vec<id::EdgeId>> = HashMap::new();
        self.edges.iter().for_each(|(mesh_id, edge_id)| {
            edges_by_mesh.entry(*mesh_id).or_default().push(*edge_id);
        });

        // Clear the cuts for each mesh in reverse order
        edges_by_mesh.iter().for_each(|(mesh_id, edge_ids)| {
            if let Some(mesh) = state.meshes.get_mut(*mesh_id) {
                edge_ids.iter().rev().for_each(|e_id| mesh.clear_cut(e_id, true));
            }
        });
        Ok(())
    }
}
