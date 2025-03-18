use crate::event::{self, EventHandler};

#[derive(Debug, Default, Copy, Clone)]
pub struct Controller2D {}

impl EventHandler for Controller2D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        // First, call any active tool's event handler. If it returns "true",
        // then do not process the event.

        // If no tool took the event, pass it to the camera.
        match ev {
            // event::UserEvent::Mouse(ev) => match ev {
            //     event::MouseEvent::Enter => todo!(),
            //     event::MouseEvent::Exit => todo!(),
            //     event::MouseEvent::Move { x, y } => todo!(),
            // },
            event::UserEvent::MouseWheel { dx, dy } => {
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
