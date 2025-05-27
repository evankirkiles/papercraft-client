use cgmath::{SquareMatrix, Transform};

use super::ToolContext;

/// Transforms the selected pieces by this amount.
#[derive(Debug, Copy, Clone)]
pub struct TransformTool {
    ctx: ToolContext,
    transform: cgmath::Matrix4<f32>,
}

impl TransformTool {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx, transform: cgmath::Matrix4::identity() }
    }

    pub fn translate(&mut self, state: &mut crate::State, dx: f32, dy: f32) {
        let up = cgmath::Vector3::unit_y();
        let right = cgmath::Vector3::unit_x();

        // --- 3. Convert dx/dy to world space ---
        // Assumes dx/dy are in NDC or screen-pixel-relative scale; tweak the scale factor if needed.
        let zoom = state.viewport_2d.camera.zoom;
        let height = self.ctx.viewport.height;
        // Uniform world-space scale per pixel, based on viewport height and zoom
        let world_per_pixel = 2.0 / height / zoom;
        let scale = world_per_pixel * self.ctx.dpr;
        let delta = right * dx * scale + up * -dy * scale;

        // --- 4. Create a translation matrix and apply it ---
        let translation_diff = cgmath::Matrix4::from_translation(delta);
        self.update(state, translation_diff);
    }

    pub fn reset(&mut self, state: &mut crate::State) {
        let diff = self.transform.inverse_transform().unwrap();
        self.update(state, diff);
    }

    fn update(&mut self, state: &mut crate::State, diff: cgmath::Matrix4<f32>) {
        state.selection.pieces.iter().for_each(|(m_id, p_id)| {
            let mesh = state.meshes.get_mut(m_id).unwrap();
            mesh.transform_piece(*p_id, diff);
        });
        self.transform = diff * self.transform;
    }
}
