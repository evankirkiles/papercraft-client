use cgmath::Transform;
use serde::{Deserialize, Serialize};

use crate::{id, MeshId};

use super::{Command, CommandError};

/// A modification of the current select state. Because there are many possible
/// side effects of these types of commands, we simply store before / after
/// snapshots of the select state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformPiecesCommand {
    pub pieces: Vec<(MeshId, id::PieceId)>,
    pub delta: cgmath::Matrix4<f32>,
}

impl Command for TransformPiecesCommand {
    fn execute(&self, state: &mut crate::State) -> Result<(), CommandError> {
        self.pieces.iter().for_each(|(m_id, p_id)| {
            state.meshes.get_mut(*m_id).unwrap().transform_piece(*p_id, self.delta);
        });
        Ok(())
    }

    fn rollback(&self, state: &mut crate::State) -> Result<(), CommandError> {
        let delta_inverse = self.delta.inverse_transform().unwrap();
        self.pieces.iter().for_each(|(m_id, p_id)| {
            state.meshes.get_mut(*m_id).unwrap().transform_piece(*p_id, delta_inverse);
        });
        Ok(())
    }
}
