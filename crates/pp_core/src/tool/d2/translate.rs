use cgmath::{Matrix4, SquareMatrix, Transform};

use crate::{id, tool::ToolContext, PhysicalPosition};

/// Translates the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct TranslateTool {
    /// How many world units each pixel corresponds to
    pub units_per_pixel: f32,
    /// The position of the mouse translations will be relative to
    pub start_pos: Option<PhysicalPosition<f32>>,
    /// Which pieces are being affected by this translation
    pub pieces: Vec<(id::MeshId, id::PieceId)>,
    /// The amount each piece is translated by
    pub transform: cgmath::Matrix4<f32>,
}

impl TranslateTool {
    pub fn new(state: &crate::State, ctx: ToolContext) -> Self {
        Self {
            units_per_pixel: ctx.dpr * 2.0 / ctx.viewport.height / state.viewport_2d.camera.zoom,
            start_pos: None,
            pieces: state.get_selected_pieces(),
            transform: cgmath::Matrix4::identity(),
        }
    }

    pub fn update(&mut self, state: &mut crate::State, pos: Option<PhysicalPosition<f32>>) {
        let (Some(start_pos), Some(pos)) = (self.start_pos, pos) else {
            self.start_pos = pos;
            return;
        };
        let (dx, dy) = (pos.x - start_pos.x, pos.y - start_pos.y);
        let translation = cgmath::Vector3::new(dx, -dy, 0.0) * self.units_per_pixel;
        self.apply(state, cgmath::Matrix4::from_translation(translation));
    }

    pub fn reset(&mut self, state: &mut crate::State) {
        self.apply(state, Matrix4::identity());
    }

    fn apply(&mut self, state: &mut crate::State, transform: cgmath::Matrix4<f32>) {
        let diff = transform * self.transform.inverse_transform().unwrap();
        self.pieces.iter().for_each(|(m_id, p_id)| {
            let mesh = state.meshes.get_mut(m_id).unwrap();
            mesh.transform_piece(*p_id, diff);
        });
        self.transform = transform;
    }
}
