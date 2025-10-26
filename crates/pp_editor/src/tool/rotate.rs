use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Rad, SquareMatrix, Transform};
use pp_core::{id, transform_pieces::TransformPiecesCommand, MeshId};

use crate::viewport::{cutting::CuttingViewport, ViewportBounds};

use super::ToolCreationError;

/// Transforms the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct RotateTool {
    /// The world position which the pieces will be rotated around.
    pub center_pos_world: cgmath::Point3<f32>,
    /// The screen position which we will measure rotations against.
    pub center_pos: cgmath::Point2<f32>,
    /// The beginning position of the tool
    pub start_pos: Option<cgmath::Point2<f32>>,
    /// The position of the mouse rotations will be relative to
    pub curr_pos: Option<cgmath::Point2<f32>>,

    /// Which pieces are being affected by this translation
    pub pieces: Vec<(MeshId, id::FaceId)>,
    /// The amount each piece is translated by
    pub transform: cgmath::Matrix4<f32>,
    /// Whether or not the tool's internal state has changed
    pub is_dirty: bool,
}

impl CuttingViewport {
    pub fn create_tool_rotate(
        &self,
        state: &pp_core::State,
        bounds: &ViewportBounds,
    ) -> Result<RotateTool, ToolCreationError> {
        let pieces = state.get_selected_pieces();
        if pieces.is_empty() {
            return Err(ToolCreationError::NoSelection);
        };

        // Get the center position of all the selected items
        let (center_pos_world, center_pos_screen) =
            self.get_center(state, bounds, &pieces).map_err(|()| ToolCreationError::NoSelection)?;

        Ok(RotateTool {
            center_pos_world,
            center_pos: center_pos_screen,
            start_pos: None,
            curr_pos: None,
            pieces,
            transform: cgmath::Matrix4::identity(),
            is_dirty: true,
        })
    }
}

impl RotateTool {
    pub fn update(&mut self, state: &mut pp_core::State, pos: Option<cgmath::Point2<f32>>) {
        self.curr_pos = pos;
        self.is_dirty = true;
        // Ensure that curr_pos has a value
        let Some(pos) = self.curr_pos else {
            return;
        };

        // Ensure that start pos has also been set
        let Some(start_pos) = self.start_pos else {
            self.start_pos = Some(pos);
            return;
        };

        // Calculate angle between the two mouse positions around the center pos
        let from: cgmath::Vector2<_> = start_pos - self.center_pos;
        let to: cgmath::Vector2<_> = pos - self.center_pos;
        let cross = from.x * to.y - from.y * to.x;
        let angle = Rad(cross.atan2(from.dot(to)));

        // Build rotation matrix around center_pos_world, around Z axis (assuming screen-aligned rotation)
        let rotation = cgmath::Matrix4::from_translation(self.center_pos_world.to_vec())
            * cgmath::Matrix4::from_angle_z(-angle)
            * cgmath::Matrix4::from_translation(-self.center_pos_world.to_vec());
        self.apply(state, rotation);
    }

    pub fn cancel(&mut self, state: &mut pp_core::State) {
        self.apply(state, Matrix4::identity());
    }

    fn apply(&mut self, state: &mut pp_core::State, transform: cgmath::Matrix4<f32>) {
        let diff = transform * self.transform.inverse_transform().unwrap();
        self.pieces.iter().for_each(|(m_id, f_id)| {
            state.meshes.get_mut(*m_id).unwrap().transform_piece(f_id, diff);
        });
        self.transform = transform;
    }
}

impl From<RotateTool> for TransformPiecesCommand {
    fn from(val: RotateTool) -> Self {
        Self { pieces: val.pieces, delta: val.transform }
    }
}
