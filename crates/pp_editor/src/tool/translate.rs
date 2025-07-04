use cgmath::{Matrix4, SquareMatrix, Transform};
use pp_core::{id, transform_pieces::TransformPiecesCommand, MeshId};

use crate::viewport::{cutting::CuttingViewport, ViewportBounds};

use super::ToolCreationError;

/// Translates the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct TranslateTool {
    /// How many world units each pixel corresponds to
    pub units_per_pixel: f32,
    /// The position of the mouse translations will be relative to
    pub start_pos: Option<cgmath::Point2<f32>>,
    /// Which pieces are being affected by this translation
    pub pieces: Vec<(MeshId, id::PieceId)>,
    /// The amount each piece is translated by
    pub transform: cgmath::Matrix4<f32>,
}

impl CuttingViewport {
    pub fn create_tool_translate(
        &self,
        state: &pp_core::State,
        bounds: &ViewportBounds,
    ) -> Result<TranslateTool, ToolCreationError> {
        let pieces = state.get_selected_pieces();
        if pieces.is_empty() {
            return Err(ToolCreationError::NoSelection);
        };
        Ok(TranslateTool {
            pieces,
            units_per_pixel: bounds.dpr * 2.0 / bounds.area.height / self.camera.zoom,
            start_pos: None,
            transform: cgmath::Matrix4::identity(),
        })
    }
}

impl TranslateTool {
    pub fn update(&mut self, state: &mut pp_core::State, pos: Option<cgmath::Point2<f32>>) {
        let (Some(start_pos), Some(pos)) = (self.start_pos, pos) else {
            self.start_pos = pos;
            return;
        };
        let (dx, dy) = (pos.x - start_pos.x, pos.y - start_pos.y);
        let translation = cgmath::Vector3::new(dx, -dy, 0.0) * self.units_per_pixel;
        self.apply(state, cgmath::Matrix4::from_translation(translation));
    }

    pub fn cancel(&mut self, state: &mut pp_core::State) {
        self.apply(state, Matrix4::identity());
    }

    fn apply(&mut self, state: &mut pp_core::State, transform: cgmath::Matrix4<f32>) {
        let diff = transform * self.transform.inverse_transform().unwrap();
        self.pieces.iter().copied().for_each(|(m_id, p_id)| {
            let mesh = state.meshes.get_mut(m_id).unwrap();
            mesh.transform_piece(p_id, diff);
        });
        self.transform = transform;
    }
}

impl From<TranslateTool> for TransformPiecesCommand {
    fn from(val: TranslateTool) -> Self {
        Self { pieces: val.pieces, delta: val.transform }
    }
}
