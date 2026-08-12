use serde::{Deserialize, Serialize};

use crate::MeshId;

use super::{Command, CommandError};

/// Applies an incremental uniform scale factor to one or more meshes, across
/// all axes. Affects each mesh's own geometry as well as its derived pieces.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScaleMeshCommand {
    pub meshes: Vec<MeshId>,
    pub factor: f32,
}

impl Command for ScaleMeshCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.meshes.iter().for_each(|m_id| {
            state.meshes.get_mut(*m_id).unwrap().scale_mesh(self.factor);
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        let factor_inverse = 1.0 / self.factor;
        self.meshes.iter().for_each(|m_id| {
            state.meshes.get_mut(*m_id).unwrap().scale_mesh(factor_inverse);
        });
        Ok(())
    }
}
