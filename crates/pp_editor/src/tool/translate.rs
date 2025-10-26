use cgmath::{Matrix4, SquareMatrix, Transform};
use pp_core::{id, transform_pieces::TransformPiecesCommand, MeshId};

use crate::viewport::{cutting::CuttingViewport, ViewportBounds};

use super::ToolCreationError;

#[derive(Debug, Clone, Copy)]
pub enum TranslateAxisLock {
    X,
    Y,
}

/// Translates the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct TranslateTool {
    /// How many world units each pixel corresponds to
    pub units_per_pixel: f32,
    /// The center position of the entire selection
    pub center_pos: cgmath::Point2<f32>,
    /// The position of the mouse translations will be relative to
    pub start_pos: Option<cgmath::Point2<f32>>,
    /// The current position of the mouse
    pub curr_pos: Option<cgmath::Point2<f32>>,

    /// Which pieces are being affected by this translation
    pub pieces: Vec<(MeshId, id::FaceId)>,
    /// The amount each piece is translated by
    pub transform: cgmath::Matrix4<f32>,
    /// If true, the transformation is locked to this axis
    pub axis_lock: Option<TranslateAxisLock>,
    /// Whether or not the tool's internal state has changed
    pub is_dirty: bool,
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

        // Get the center position of all the selected items
        let (_, center_pos) =
            self.get_center(state, bounds, &pieces).map_err(|()| ToolCreationError::NoSelection)?;

        Ok(TranslateTool {
            pieces,
            units_per_pixel: bounds.dpr * 2.0 / bounds.area.height / self.camera.zoom,
            center_pos,
            start_pos: None,
            curr_pos: None,
            axis_lock: None,
            transform: cgmath::Matrix4::identity(),
            is_dirty: true,
        })
    }
}

impl TranslateTool {
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

        let (dx, dy) = (pos.x - start_pos.x, pos.y - start_pos.y);
        let (dx, dy) = match self.axis_lock {
            Some(TranslateAxisLock::X) => (dx, 0.0),
            Some(TranslateAxisLock::Y) => (0.0, dy),
            None => (dx, dy),
        };
        let translation = cgmath::Vector3::new(dx, -dy, 0.0) * self.units_per_pixel;
        self.apply(state, cgmath::Matrix4::from_translation(translation));
    }

    pub fn toggle_x_lock(&mut self, state: &mut pp_core::State) {
        self.axis_lock = match &self.axis_lock {
            Some(TranslateAxisLock::X) => None,
            _ => Some(TranslateAxisLock::X),
        };
        self.update(state, self.curr_pos);
    }

    pub fn toggle_y_lock(&mut self, state: &mut pp_core::State) {
        self.axis_lock = match &self.axis_lock {
            Some(TranslateAxisLock::Y) => None,
            _ => Some(TranslateAxisLock::Y),
        };
        self.update(state, self.curr_pos);
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

impl From<TranslateTool> for TransformPiecesCommand {
    fn from(val: TranslateTool) -> Self {
        Self { pieces: val.pieces, delta: val.transform }
    }
}
