use pp_editor::tool::Tool;

pub mod rotate;
pub mod select_box;
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
        }
    }
}
