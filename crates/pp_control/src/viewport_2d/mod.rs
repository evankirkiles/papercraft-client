use crate::{
    event::{self, EventHandler},
    keyboard, tool,
};

#[derive(Debug, Clone)]
pub enum Controller2DTool {
    Select(tool::SelectTool),
    Translate(pp_core::tool::d2::TranslateTool),
    Rotate(pp_core::tool::d2::RotateTool),
}

impl Default for Controller2DTool {
    fn default() -> Self {
        Self::Select(tool::SelectTool::default())
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Controller2D {
    tool: Controller2DTool,
}

/// Builds a tool's context from an event context
fn get_tool_ctx(ctx: &event::EventContext) -> pp_core::tool::ToolContext {
    let renderer = ctx.renderer.borrow();
    let size = renderer.as_ref().unwrap().curr_size();
    pp_core::tool::ToolContext { viewport: size, dpr: 2.0 }
}

impl Controller2D {
    fn handle_event_with_tool(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        let res = match self.tool {
            Controller2DTool::Select(ref mut tool) => tool.handle_event(ctx, ev),
            Controller2DTool::Translate(ref mut tool) => tool.handle_event(ctx, ev),
            Controller2DTool::Rotate(ref mut tool) => tool.handle_event(ctx, ev),
        };
        if res.is_ok_and(|e| e.clear_tool) {
            self.tool = Default::default()
        }
        res
    }
}

impl event::EventHandler for Controller2D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        // First, call any active tool's event handler. If it returns "true",
        // then do not process the event.
        let res = self.handle_event_with_tool(ctx, ev)?;
        if res.stop_propagation {
            return Ok(res);
        }

        // If no tool took the event, pass it to the camera.
        use event::UserEvent;
        match ev {
            UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Character(char),
            )) => match char.as_str() {
                "KeyG" => {
                    let state = ctx.state.borrow();
                    self.tool = Controller2DTool::Translate(pp_core::tool::d2::TranslateTool::new(
                        &state,
                        get_tool_ctx(ctx),
                    ));
                }
                "KeyR" => {
                    let state = ctx.state.borrow();
                    let tool = pp_core::tool::d2::RotateTool::new(&state, get_tool_ctx(ctx));
                    if let Ok(tool) = tool {
                        self.tool = Controller2DTool::Rotate(tool);
                    }
                }
                _ => {}
            },
            UserEvent::MouseWheel { dx, dy } => {
                let mut state = ctx.state.borrow_mut();
                if ctx.modifiers.super_pressed() {
                    state.viewport_2d.camera.dolly(*dy * 0.5);
                } else {
                    state.viewport_2d.camera.pan(*dx, *dy);
                }
            }
            _ => {}
        };
        Ok(event::InternalEventHandleSuccess::stop_internal_propagation())
    }
}
