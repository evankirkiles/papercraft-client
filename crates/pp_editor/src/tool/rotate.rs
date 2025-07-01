use cgmath::{ElementWise, EuclideanSpace, InnerSpace, Matrix4, Rad, SquareMatrix, Transform};
use pp_core::{id, transform_pieces::TransformPiecesCommand};

use crate::viewport::{camera::Camera, cutting::CuttingViewport, ViewportBounds};

use super::ToolCreationError;

/// Transforms the selected pieces by this amount.
#[derive(Debug, Clone)]
pub struct RotateTool {
    /// The world position which the pieces will be rotated around.
    pub center_pos_world: cgmath::Point3<f32>,
    /// The screen position which we will measure rotations against.
    pub center_pos_screen: cgmath::Point2<f32>,
    /// The position of the mouse rotations will be relative to
    pub start_pos: Option<cgmath::Point2<f32>>,
    /// Which pieces are being affected by this translation
    pub pieces: Vec<(id::MeshId, id::PieceId)>,
    /// The amount each piece is translated by
    pub transform: cgmath::Matrix4<f32>,
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

        // Calculate the average unfolded world position of the selected faces' vertices
        let vert_pos: Vec<_> = pieces
            .iter()
            .flat_map(|(m_id, p_id)| {
                let mesh = &state.meshes[m_id];
                mesh.iter_piece_faces_unfolded(*p_id)
                    .map(|face| ((*m_id, face.f), mesh[*p_id].transform, face))
            })
            .filter(|(id, _, _)| state.selection.faces.contains(id))
            .flat_map(|((m_id, _), piece_affine, item)| {
                let mesh = &state.meshes[&m_id];
                mesh.iter_face_loops(item.f).map(move |l| {
                    piece_affine.transform_point(
                        item.affine.transform_point(cgmath::Point3::from(mesh[mesh[l].v].po)),
                    )
                })
            })
            .collect();
        let count = vert_pos.len();
        let vert_sum = vert_pos.into_iter().reduce(move |p_a, p_b| p_a.add_element_wise(p_b));
        let Some(vert_sum) = vert_sum else { return Err(ToolCreationError::NoSelection) };
        let center_pos_world = vert_sum / count as f32;

        // Now, get the screen coordinates  of the center to determine our rotation axis
        let view_proj = self.camera.view_proj(bounds.area.into());
        let center_pos_ndc = view_proj.transform_point(center_pos_world);
        let center_pos_screen =
            bounds.area.ndc(cgmath::Point2::new(center_pos_ndc.x, center_pos_ndc.y));

        Ok(RotateTool {
            center_pos_world,
            center_pos_screen,
            start_pos: None,
            pieces,
            transform: cgmath::Matrix4::identity(),
        })
    }
}

impl RotateTool {
    pub fn update(&mut self, state: &mut pp_core::State, pos: Option<cgmath::Point2<f32>>) {
        let (Some(start_pos), Some(pos)) = (self.start_pos, pos) else {
            self.start_pos = pos;
            return;
        };

        // Calculate angle between the two mouse positions around the center pos
        let from: cgmath::Vector2<_> = (start_pos - self.center_pos_screen).into();
        let to: cgmath::Vector2<_> = (pos - self.center_pos_screen).into();
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
        self.pieces.iter().for_each(|(m_id, p_id)| {
            let mesh = state.meshes.get_mut(m_id).unwrap();
            mesh.transform_piece(*p_id, diff);
        });
        self.transform = transform;
    }
}

impl From<RotateTool> for TransformPiecesCommand {
    fn from(val: RotateTool) -> Self {
        Self { pieces: val.pieces, delta: val.transform }
    }
}
