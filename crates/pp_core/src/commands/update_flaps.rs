use std::collections::HashMap;

use crate::{
    id,
    mesh::{edge::EdgeCut, MeshElementType},
};

use super::{Command, CommandError};

/// A modification of the current select state. Because there are many possible
/// side effects of these types of commands, we simply store before / after
/// snapshots of the select state.
#[derive(Clone, Debug)]
pub struct UpdateFlapsCommand {
    pub edges: Vec<(id::MeshId, id::EdgeId)>,
    pub before: HashMap<(id::MeshId, id::EdgeId), EdgeCut>,
    pub after: HashMap<(id::MeshId, id::EdgeId), EdgeCut>,
}

impl UpdateFlapsCommand {
    /// Entirely swaps which edge the flap is on
    pub fn swap_flaps(state: &mut crate::State) -> Self {
        let cut_edges: Vec<_> = state
            .selection
            .edges
            .iter()
            .copied()
            .filter(|id| state.meshes[&id.0][id.1].cut.is_some())
            .collect();
        // Build up the previous history around those edges. What were the
        // cut states, what were the existing pieces, etc.
        let mut before: HashMap<(id::MeshId, id::EdgeId), EdgeCut> = HashMap::new();
        let mut after: HashMap<(id::MeshId, id::EdgeId), EdgeCut> = HashMap::new();
        cut_edges.iter().for_each(|id| {
            before.insert(*id, state.meshes[&id.0][id.1].cut.unwrap());
            state.swap_edge_flap(id);
            after.insert(*id, state.meshes[&id.0][id.1].cut.unwrap());
        });
        Self { edges: cut_edges, before, after }
    }
}

impl Command for UpdateFlapsCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.edges.iter().for_each(|id| {
            let mesh = state.meshes.get_mut(&id.0).unwrap();
            mesh[id.1].cut = Some(self.after[id]);
            mesh.elem_dirty |= MeshElementType::FLAPS;
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.edges.iter().for_each(|id| {
            let mesh = state.meshes.get_mut(&id.0).unwrap();
            mesh[id.1].cut = Some(self.before[id]);
            mesh.elem_dirty |= MeshElementType::FLAPS;
        });
        Ok(())
    }
}
