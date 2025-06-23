use crate::{
    event::{self, EventHandler, MouseButton, PointerEvent},
    keyboard,
};

impl EventHandler for pp_core::tool::d2::RotateTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        event: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        let state = &mut ctx.state.borrow_mut();
        match event {
            // On mouse move, rotate the piece accordingly
            event::UserEvent::Pointer(e) => match e {
                PointerEvent::Exit => self.update(state, None),
                PointerEvent::Move(pos) => self.update(state, Some(*pos)),
                _ => (),
            },
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => match button {
                // LMB "accepts" the tool changes, removing the translate tool and
                // adding an entry onto the history stack for undoing the changes
                MouseButton::Left => {
                    ctx.history
                        .borrow_mut()
                        .add(pp_core::CommandType::TransformPieces(self.clone().into()));
                    return Ok(event::InternalEventHandleSuccess::clear_tool());
                }
                // RMB cancels the tool
                MouseButton::Right => {
                    self.reset(state);
                    return Ok(event::InternalEventHandleSuccess::clear_tool());
                }
                _ => (),
            },
            // ESC also cancels the tool
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Named(keyboard::NamedKey::Escape),
            )) => {
                self.reset(state);
                return Ok(event::InternalEventHandleSuccess::clear_tool());
            }
            _ => (),
        };
        Ok(event::InternalEventHandleSuccess::stop_propagation())
    }
}
