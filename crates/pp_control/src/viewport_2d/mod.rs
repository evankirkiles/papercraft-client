use crate::event;

mod tool_select;

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Controller2D {
    // The active tool.
    tool: Controller2DTool,
}

impl event::EventHandler for Controller2D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        // First, call any active tool's event handler. If it returns "true",
        // then do not process the event.
        if self.tool.handle_event(ctx, ev)? == event::EventHandleSuccess::StopPropagation {
            return Ok(event::EventHandleSuccess::StopPropagation);
        }

        // If no tool took the event, pass it to the camera.
        use event::UserEvent;
        match ev {
            // event::UserEvent::Mouse(ev) => match ev {
            //     event::MouseEvent::Enter => todo!(),
            //     event::MouseEvent::Exit => todo!(),
            //     event::MouseEvent::Move { x, y } => todo!(),
            // },
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
        Ok(event::EventHandleSuccess::StopPropagation)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Controller2DTool {
    Select(tool_select::SelectTool),
}

impl event::EventHandler for Controller2DTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        match *self {
            Controller2DTool::Select(ref mut tool) => tool.handle_event(ctx, ev),
        }
    }
}

impl Default for Controller2DTool {
    fn default() -> Self {
        Self::Select(tool_select::SelectTool::default())
    }
}
