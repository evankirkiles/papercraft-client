use crate::{
    event::{self, EventHandler, MouseButton, PointerEvent},
    keyboard,
};

impl EventHandler for pp_editor::tool::TranslateTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        event: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let state = &mut ctx.state.borrow_mut();
        match event {
            // On mouse move, translate the piece accordingly
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
                    return Ok(event::EventHandleSuccess::set_tool(None));
                }
                // RMB cancels the tool
                MouseButton::Right => {
                    self.cancel(state);
                    return Ok(event::EventHandleSuccess::set_tool(None));
                }
                _ => (),
            },
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
                // ESC also cancels the tool
                keyboard::Key::Named(keyboard::NamedKey::Escape) => {
                    self.cancel(state);
                    return Ok(event::EventHandleSuccess::set_tool(None));
                }
                keyboard::Key::Character(char) => match char.as_str() {
                    // X: Toggle X axis lock
                    "KeyX" => self.toggle_x_lock(state),
                    // Y: Toggle Y axis lock
                    "KeyY" => self.toggle_y_lock(state),
                    // TODO: Space: Turn on "move" mode (move start_pos)
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        };
        Ok(event::EventHandleSuccess::stop_propagation())
    }
}
