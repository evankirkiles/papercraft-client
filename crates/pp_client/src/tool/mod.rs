use pp_editor::tool::Tool;

pub mod rotate;
pub mod select_box;
pub mod translate;

use crate::EventHandler;

impl EventHandler for Tool {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        ev: &crate::UserEvent,
    ) -> Option<Result<crate::event::EventHandleSuccess, crate::event::EventHandleError>> {
        match self {
            Tool::Translate(tool) => tool.handle_event(ctx, ev),
            Tool::Rotate(tool) => tool.handle_event(ctx, ev),
            Tool::SelectBox(tool) => tool.handle_event(ctx, ev),
        }
    }
}
