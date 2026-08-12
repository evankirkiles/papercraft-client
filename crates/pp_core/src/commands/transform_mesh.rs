use cgmath::Transform;
use serde::{Deserialize, Serialize};

use crate::MeshId;

use super::{Command, CommandError};

/// Applies an incremental affine transform (translate + rotate) to one or
/// more meshes. Only affects the whole-mesh 3D view; pieces are unaffected.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformMeshCommand {
    pub meshes: Vec<MeshId>,
    pub delta: cgmath::Matrix4<f32>,
}

impl Command for TransformMeshCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.meshes.iter().for_each(|m_id| {
            state.meshes.get_mut(*m_id).unwrap().transform_mesh(self.delta);
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        let delta_inverse = self.delta.inverse_transform().unwrap();
        self.meshes.iter().for_each(|m_id| {
            state.meshes.get_mut(*m_id).unwrap().transform_mesh(delta_inverse);
        });
        Ok(())
    }
}
