use serde::{Deserialize, Serialize};

use crate::{
    id,
    mesh::{cut::FlapPosition, MeshElementType},
    MeshId,
};

use super::{Command, CommandError};

/// A modification of the current select state. Because there are many possible
/// side effects of these types of commands, we simply store before / after
/// snapshots of the select state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateFlapsCommand {
    pub before: Vec<((MeshId, id::EdgeId), FlapPosition)>,
    pub after: Vec<((MeshId, id::EdgeId), FlapPosition)>,
}

impl UpdateFlapsCommand {
    /// Entirely swaps which edge the flap is on
    pub fn swap_flaps(state: &mut crate::State) -> Self {
        let edges: Vec<_> = state
            .selection
            .edges
            .iter()
            .copied()
            .filter(|id| state.meshes[id.0].cuts.get(&id.1).is_some_and(|cut| !cut.is_dead))
            .collect();
        // Build up the previous history around those edges. What were the
        // cut states, what were the existing pieces, etc.
        let mut before: Vec<_> = Vec::new();
        let mut after: Vec<_> = Vec::new();
        edges.iter().copied().for_each(|id| {
            let mesh = &mut state.meshes[id.0];
            let Some(flap_position) = mesh.cuts.get_mut(&id.1).map(|cut| cut.flap_position) else {
                return;
            };
            before.push((id, flap_position));
            let new_flap = match flap_position {
                FlapPosition::FirstFace => FlapPosition::SecondFace,
                FlapPosition::SecondFace => FlapPosition::FirstFace,
                FlapPosition::BothFaces => FlapPosition::BothFaces,
                FlapPosition::None => FlapPosition::None,
            };
            mesh.set_cut_flap(id.1, new_flap);
            after.push((id, new_flap));
        });
        Self { before, after }
    }
}

impl Command for UpdateFlapsCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.after.iter().for_each(|(id, flap_position)| {
            let mesh = state.meshes.get_mut(id.0).unwrap();
            mesh.set_cut_flap(id.1, *flap_position);
            mesh.elem_dirty |= MeshElementType::FLAPS;
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.before.iter().for_each(|(id, flap_position)| {
            let mesh = state.meshes.get_mut(id.0).unwrap();
            mesh.set_cut_flap(id.1, *flap_position);
            mesh.elem_dirty |= MeshElementType::FLAPS;
        });
        Ok(())
    }
}
