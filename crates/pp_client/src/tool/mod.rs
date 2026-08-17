use pp_core::{
    id::{self, Id},
    select::SelectionActionType,
    MeshId,
};
use pp_draw::select::{PixelData, SelectionMask};
use pp_editor::{state::SelectionMode, tool::Tool};
use slotmap::KeyData;

pub mod rotate;
pub mod select_box;
pub mod select_paint;
pub mod translate;

use crate::EventHandler;

impl EventHandler for Tool {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        editor_state: &mut pp_editor::state::EditorState,
        ev: &crate::UserEvent,
    ) -> Option<Result<crate::event::EventHandleSuccess, crate::event::EventHandleError>> {
        match self {
            Tool::Translate(tool) => tool.handle_event(ctx, editor_state, ev),
            Tool::Rotate(tool) => tool.handle_event(ctx, editor_state, ev),
            Tool::SelectBox(tool) => tool.handle_event(ctx, editor_state, ev),
            Tool::SelectPaint(tool) => tool.handle_event(ctx, editor_state, ev),
        }
    }
}

/// The elements the select buffer should be populated with for a given
/// selection granularity.
pub fn selection_mask(mode: &SelectionMode) -> SelectionMask {
    match mode {
        SelectionMode::Vert => SelectionMask::VERTS,
        SelectionMode::Edge => SelectionMask::EDGES,
        SelectionMode::Face => SelectionMask::FACES,
        SelectionMode::Piece => SelectionMask::PIECES,
    }
}

/// Applies a selection action to the single element a select-buffer pixel
/// refers to, interpreting the pixel according to the queried mask.
pub fn apply_pixel(
    state: &mut pp_core::State,
    mask: SelectionMask,
    pixel: &PixelData,
    action: SelectionActionType,
    activate: bool,
) {
    let mesh_id: MeshId = KeyData::from_ffi(pixel.mesh_id).into();
    match mask {
        SelectionMask::VERTS => {
            let vert_id = id::VertexId::new(pixel.el_id);
            state.select_vert(&(mesh_id, vert_id), action, activate);
        }
        SelectionMask::EDGES => {
            let edge_id = id::EdgeId::new(pixel.el_id);
            state.select_edge(&(mesh_id, edge_id), action, activate, true);
        }
        SelectionMask::FACES => {
            let face_id = id::FaceId::new(pixel.f_id);
            state.select_face(&(mesh_id, face_id), action, activate, true);
        }
        SelectionMask::PIECES => {
            let face_id = id::FaceId::new(pixel.f_id);
            let p_id = state.meshes[mesh_id].faces[face_id.to_usize()].p;
            if let Some(p_id) = p_id {
                state.select_piece(&(mesh_id, p_id), action);
            }
        }
        _ => {}
    }
}
