use crate::event::{self, EventHandler};

mod tool_select;

/// The Controller3D modifies state based on input to the 3D viewport pane.
#[derive(Debug, Default, Clone, Copy)]
pub struct Controller3D {
    // The active tool.
    tool: Controller3DTool,
}

impl EventHandler for Controller3D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        // First, call any active tool's event handler, potentially ending propagation
        if self.tool.handle_event(ctx, ev)? == event::EventHandleSuccess::StopPropagation {
            return Ok(event::EventHandleSuccess::StopPropagation);
        }

        // If no tool took the event, pass it to the camera.
        #[allow(clippy::single_match)]
        match ev {
            event::UserEvent::MouseWheel { dx, dy } => {
                let mut state = ctx.state.borrow_mut();
                if ctx.modifiers.shift_pressed() {
                    state.viewport_3d.camera.pan(*dx, *dy);
                } else if ctx.modifiers.alt_pressed() {
                    state.viewport_3d.camera.dolly(*dy * 0.5);
                } else {
                    state.viewport_3d.camera.orbit(*dx, *dy);
                }
            }
            _ => {}
        };
        Ok(event::EventHandleSuccess::StopPropagation)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Controller3DTool {
    Select(tool_select::SelectTool),
}

impl Default for Controller3DTool {
    fn default() -> Self {
        Self::Select(tool_select::SelectTool::default())
    }
}

impl EventHandler for Controller3DTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        match *self {
            Controller3DTool::Select(ref mut tool) => tool.handle_event(ctx, ev),
        }
    }
}
