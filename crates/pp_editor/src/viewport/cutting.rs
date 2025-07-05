use cgmath::{ElementWise, Transform};
use pp_core::{id::PieceId, MeshId};
use serde::{Deserialize, Serialize};

use super::{
    camera::{orthographic::OrthographicCamera, Camera},
    ViewportBounds,
};

/// A viewport for "cutting", meaning a 2D orthographic view of pieces, layout
/// elements like images / text, and pages.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CuttingViewport {
    pub camera: OrthographicCamera,
}

impl CuttingViewport {
    /// Gets the center position between all the selected faces in the provided pieces
    pub fn get_center(
        &self,
        state: &pp_core::State,
        bounds: &ViewportBounds,
        pieces: &[(MeshId, PieceId)],
    ) -> Result<(cgmath::Point3<f32>, cgmath::Point2<f32>), ()> {
        let vert_pos: Vec<_> = pieces
            .iter()
            .copied()
            .flat_map(|(m_id, p_id)| {
                let mesh = &state.meshes[m_id];
                mesh.iter_piece_faces_unfolded(p_id)
                    .map(move |face| ((m_id, face.f), mesh[p_id].transform, face))
            })
            .filter(|(id, _, _)| state.selection.faces.contains(id))
            .flat_map(|((m_id, _), piece_affine, item)| {
                let mesh = &state.meshes[m_id];
                mesh.iter_face_loops(item.f).map(move |l| {
                    piece_affine.transform_point(
                        item.affine.transform_point(cgmath::Point3::from(mesh[mesh[l].v].po)),
                    )
                })
            })
            .collect();
        let count = vert_pos.len();
        let vert_sum = vert_pos.into_iter().reduce(move |p_a, p_b| p_a.add_element_wise(p_b));
        let Some(vert_sum) = vert_sum else { return Err(()) };
        let center_pos_world = vert_sum / count as f32;

        // Now, get the screen coordinates  of the center to determine our rotation axis
        let view_proj = self.camera.view_proj(bounds.area.into());
        let center_pos_ndc = view_proj.transform_point(center_pos_world);
        let center_pos_screen =
            bounds.area.ndc(cgmath::Point2::new(center_pos_ndc.x, center_pos_ndc.y));
        Ok((center_pos_world, center_pos_screen))
    }
}
