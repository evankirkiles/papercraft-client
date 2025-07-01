use pp_editor::tool::Tool;
use pp_editor::viewport::{cutting::CuttingViewport, ViewportBounds};

use crate::{
    event::{self, EventHandleError, EventHandleSuccess, UserEvent},
    keyboard,
};

use super::ViewportEventHandler;

impl ViewportEventHandler for CuttingViewport {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        ev: &crate::UserEvent,
        bounds: &ViewportBounds,
    ) -> Result<crate::event::EventHandleSuccess, crate::event::EventHandleError> {
        match ev {
            UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Character(char),
            )) => match char.as_str() {
                // G: Translate
                "KeyG" => {
                    return self
                        .create_tool_translate(&ctx.state.borrow(), bounds)
                        .map(|tool| EventHandleSuccess::set_tool(Some(Tool::Translate(tool))))
                        .map_err(|_| EventHandleError::default());
                }
                // R: Rotate
                "KeyR" => {
                    return self
                        .create_tool_rotate(&ctx.state.borrow(), bounds)
                        .map(|tool| EventHandleSuccess::set_tool(Some(Tool::Rotate(tool))))
                        .map_err(|_| EventHandleError::default());
                }
                _ => {}
            },
            UserEvent::MouseWheel { delta } => {
                if ctx.modifiers.super_pressed() {
                    self.camera.zoom(delta.y * 0.5);
                } else {
                    self.camera.pan(delta);
                };
            }
            _ => {}
        };
        Ok(Default::default())
    }
}
